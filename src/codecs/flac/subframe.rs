// src/codecs/flac/subframe.rs

use crate::common::{errors::AudioError, stream::Stream};

// Enum to represent the type of FLAC subframe
#[derive(Debug, PartialEq)]
pub enum SubframeType {
    Constant,
    Fixed,
    LPC,
    Verbatim,
    Unknown,
}

#[derive(Debug)]
pub struct Subframe {
    subframe_type: SubframeType,
    order: u32,     // Order of the predictor polynomial--if applicable
    data: Vec<u32>, // This is the raw data contained in the subframe
}

impl Subframe {
    // Method to read and parse a subframe from the stream
    pub fn read_next_subframe(
        stream: &mut Stream,
        bit_depth: u8,
        block_size: usize,
    ) -> Result<Self, AudioError> {
        let header_byte = stream.read_byte()?;
        let mut order = 0;
        let subframe_type = match header_byte >> 2 {
            0 => SubframeType::Constant,
            1 => SubframeType::Verbatim,
            2..=9 => {
                order = ((header_byte >> 1) - 1) as u32;
                SubframeType::Fixed
            }
            10..=31 => {
                order = ((header_byte & 0x1F) + 1) as u32;
                SubframeType::LPC
            }
            _ => SubframeType::Unknown,
        };

        println!("Subframe type: {:?}", subframe_type);
        println!("Order: {:?}", order);

        let data = match subframe_type {
            SubframeType::Constant => {
                let value = stream.read_bits(bit_depth)?;
                vec![value; block_size] // Repeat the value for each sample in the block
            }
            SubframeType::Verbatim => {
                let mut values = Vec::new();
                for _ in 0..block_size {
                    let value = stream.read_bits(bit_depth)?;
                    values.push(value);
                }
                values
            }
            SubframeType::Fixed => {
                // Read warm-up samples
                let mut warmup_samples = vec![0; order as usize];
                for sample in warmup_samples.iter_mut() {
                    *sample = stream.read_signed_bits(bit_depth)?;
                }

                // Read rice-encoded residuals
                let residuals =
                    Self::read_rice_encoded_residuals(stream, bit_depth, block_size, order)?;

                // apply predictor to calculate samples
                let samples = Self::apply_fixed_predictor(warmup_samples, residuals, order)?;

                samples.into_iter().map(|x| x as u32).collect()
            }
            SubframeType::LPC => {
                // Read warm-up samples
                let mut warmup_samples = vec![0; order as usize];
                for sample in warmup_samples.iter_mut() {
                    *sample = stream.read_signed_bits(bit_depth)?;
                }

                // read coefficients
                let precision = (stream.read_bits(4)? + 1) as u8;
                let shift = stream.read_signed_bits(5)?;
                let mut coefficients = vec![0; order as usize];
                for coefficient in coefficients.iter_mut() {
                    *coefficient = stream.read_signed_bits(precision)?;
                }

                // read rice-encoded residuals
                let residuals =
                    Self::read_rice_encoded_residuals(stream, bit_depth, block_size, order)?;

                // apply predictor to calculate samples
                let samples =
                    Self::apply_lpc_predictor(warmup_samples, coefficients, shift, residuals)?;

                samples.into_iter().map(|x| x as u32).collect()
            }
            SubframeType::Unknown => {
                return Err(AudioError::InvalidData(String::from(
                    "Invalid subframe type",
                )));
            }
        };

        Ok(Subframe {
            subframe_type,
            order,
            data,
        })
    }

    fn read_rice_encoded_residuals(
        stream: &mut Stream,
        bit_depth: u8,
        block_size: usize,
        order: u32,
    ) -> Result<Vec<i32>, AudioError> {
        // The Rice partition order is encoded in the first 4 bits of the residual section
        let partition_order = stream.read_bits(4)? as usize;
        let mut partition_count = 0;

        println!("Block size: {}", block_size);
        println!("partition order {}", partition_order);
        println!("order {}", order);

        // Calcuate the number of partitions (2^order) and samples per partition
        let num_partitions = 1 << partition_order;
        println!("num partitions {:?}", num_partitions);
        let samples_per_partition: usize = if partition_count == num_partitions {
            block_size / num_partitions - order as usize
        } else {
            block_size / num_partitions
        };

        // The remainder of the residuals are Rice-coded
        let mut residuals = Vec::new();

        for _ in 0..num_partitions {
            partition_count += 1;
            // The rice prefix is in the next bit
            let rice_prefix = stream.read_bit()?;
            let rice_parameter = match rice_prefix {
                0 => stream.read_bits(4)?, // For Rice1, parameter is 4 bits
                1 => stream.read_bits(5)?, // For Rice2, parameter is 5 bits
                _ => {
                    return Err(AudioError::InvalidData("Invalid Rice prefix".to_string()));
                }
            };

            // Read the unary-encoded residuals
            for _ in 0..samples_per_partition {
                let mut unary = 0;
                while stream.read_bit()? == 0 {
                    unary += 1;
                }
                let remainder = stream.read_bits(rice_parameter as u8)?; // read the remainder
                residuals.push(Self::decode_rice(unary, remainder));
            }
        }
        Ok(residuals)
    }

    fn decode_rice(parameter: u32, remainder: u32) -> i32 {
        // Take unary shifted by the remainder, plus the remainder itself
        ((parameter << 1) | remainder) as i32
    }

    fn apply_fixed_predictor(
        warmup_samples: Vec<i32>,
        residuals: Vec<i32>,
        order: u32,
    ) -> Result<Vec<i32>, AudioError> {
        let block_size = warmup_samples.len() + residuals.len();
        let mut samples = Vec::with_capacity(block_size);
        samples.extend(warmup_samples);

        // Calculate the predicted samples
        for i in order as usize..block_size {
            let predicted_sample = match order {
                1 => samples[i - 1],
                // 2 => 2 * samples[i - 1] - samples[i - 2],
                2 => {
                    let mul_result = samples[i - 1].checked_mul(2);
                    match mul_result {
                        Some(val) => val
                            .checked_sub(samples[i - 2])
                            .ok_or(AudioError::ArithmeticOverflow)?,
                        None => return Err(AudioError::ArithmeticOverflow),
                    }
                }
                // 3 => 3 * samples[i - 1] - 3 * samples[i - 2] + samples[i - 3],
                // 4 => 4 * samples[i - 1] - 6 * samples[i - 2] + 4 * samples[i - 3] - samples[i - 4],
                3 => {
                    let mul_3_samples_1 = samples[i - 1].checked_mul(3);
                    let mul_3_samples_2 = samples[i - 2].checked_mul(3);
                    let result = mul_3_samples_1
                        .and_then(|a| mul_3_samples_2.map(|b| a - b))
                        .and_then(|a| a.checked_add(samples[i - 3]))
                        .ok_or(AudioError::ArithmeticOverflow)?;
                    result
                }
                4 => {
                    let mul_4_samples_1 = samples[i - 1].checked_mul(4);
                    let mul_6_samples_2 = samples[i - 2].checked_mul(6);
                    let mul_4_samples_3 = samples[i - 3].checked_mul(4);
                    let result = mul_4_samples_1
                        .and_then(|a| mul_6_samples_2.map(|b| a - b))
                        .and_then(|a| mul_4_samples_3.map(|b| a + b))
                        .and_then(|a| a.checked_sub(samples[i - 4]))
                        .ok_or(AudioError::ArithmeticOverflow)?;
                    result
                }
                _ => {
                    return Err(AudioError::InvalidData(
                        "Invalid order in Fixed subframe".to_string(),
                    ))
                }
            };

            // Add the residual to get the original sample
            let sample = predicted_sample
                .checked_add(residuals[i - order as usize])
                .ok_or(AudioError::ArithmeticOverflow)?;
            samples.push(sample);
        }
        Ok(samples)
    }

    fn apply_lpc_predictor(
        warmup_samples: Vec<i32>,
        coefficients: Vec<i32>,
        shift: i32,
        residuals: Vec<i32>,
    ) -> Result<Vec<i32>, AudioError> {
        let block_size = warmup_samples.len() + residuals.len();
        let mut samples = Vec::with_capacity(block_size);
        samples.extend(warmup_samples);

        // Calculate the predicted samples
        for i in coefficients.len()..block_size {
            let mut predicted_sample: i64 = 0; // explicitly declare type as i64
            for j in 0..coefficients.len() {
                let temp = (coefficients[j] as i64)
                    .checked_mul(samples[i - j - 1] as i64)
                    .ok_or(AudioError::ArithmeticOverflow)?;
                predicted_sample = predicted_sample
                    .checked_add(temp)
                    .ok_or(AudioError::ArithmeticOverflow)?;
            }

            // Shift the prediction, with overflow protection
            let predicted_sample = predicted_sample
                .checked_shr(shift as u32)
                .ok_or(AudioError::ArithmeticOverflow)?;

            // Add the residual to get the original sample, with overflow protection
            let residual = residuals[i - coefficients.len()] as i64;
            let sample = predicted_sample
                .checked_add(residual)
                .ok_or(AudioError::ArithmeticOverflow)?;

            samples.push(sample as i32); // cast back to i32 when pushing to samples
        }
        Ok(samples)
    }

    pub fn get_data(&self) -> &[u32] {
        &self.data
    }
}
