// src/codecs/flac/frame.rs

// This module handles reading and parsing frames of data from the FLAC stream

use super::subframe::Subframe;
use crate::common::{errors::AudioError, stream::Stream};

// Struct to represent a frame of data in a FLAC stream
#[derive(Debug)]
pub struct Frame {
    sync_code: u16,                     // Synchronization code
    block_strategy: bool,               // Block strategy
    block_size: u32,                    // Block size (number of samples per channel)
    sample_rate: u32,                   // Sample rate in Hz
    channel_assignment: u8,             // Channel assignment
    bit_depth: u8,                      // Bit depth
    sample_number_or_frame_number: u64, // Sample number of first sample in frame
    crc: u16,                           // Frame CRC-16
    subframes: Vec<Subframe>,           // Subframes contained in the frame
}

impl Frame {
    // Method to read and parse a frame from the stream
    pub fn read_next_frame(stream: &mut Stream) -> Result<Self, AudioError> {
        // Implements frame parsing
        let sync_code = stream.read_bits(14)? as u16;
        if sync_code != 0x3FFE {
            return Err(AudioError::InvalidData(
                "Invalid synchronization code in FLAC frame.".to_string(),
            ));
        }

        // Read and discard 1 reserved bit
        stream.read_bit();

        // Read and discard blocking strategy bit
        let block_strategy = stream.read_bit()? != 0;

        // TODO: process block_size and sample_rate
        let block_size_code = stream.read_bits(4)?;
        let block_size = match block_size_code {
            0 => {
                return Err(AudioError::InvalidData(
                    "Reserved block size value".to_string(),
                ))
            }
            1 => 192,
            2..=5 => 576 * (1 << (block_size_code - 2)),
            6 => (stream.read_bits(8)? + 1) as u32,
            7 => (stream.read_bits(16)? + 1) as u32,
            _ => 256 * (1 << (block_size_code - 8)),
        };

        let sample_rate_code = stream.read_bits(4)?;
        let sample_rate = match sample_rate_code {
            0 => {
                return Err(AudioError::InvalidData(
                    "Sample rate must be retrieved from STREAMINFO metadata block".to_string(),
                ))
            }
            1 => 88200,
            2 => 176400,
            3 => 192000,
            4 => 8000,
            5 => 16000,
            6 => 22050,
            7 => 24000,
            8 => 32000,
            9 => 44100,
            10 => 48000,
            11 => 96000,
            12 => stream.read_bits(8)? * 1000,
            13 => stream.read_bits(16)?,
            14 => stream.read_bits(16)? * 10,
            15 => {
                return Err(AudioError::InvalidData(
                    "Invalid sample rate value".to_string(),
                ))
            }
            _ => {
                return Err(AudioError::InvalidData(
                    "Unexpected sample rate code".to_string(),
                ))
            }
        };

        // Read channel_assignment and map to a specific channel layout
        let channel_assignment = stream.read_bits(4)? as u8;

        // Read bit_depth and map it to a specific sample size
        let bit_depth_code = stream.read_bits(3)? as u8;
        let bit_depth = match bit_depth_code {
            0 => {
                return Err(AudioError::InvalidData(
                    "Bit depth must be retrieved from STREAMINFO metadata block".to_string(),
                ))
            }
            1 => 8,
            2 => 12,
            3 => {
                return Err(AudioError::InvalidData(
                    "Reserved bit depth value".to_string(),
                ))
            }
            4 => 16,
            5 => 20,
            6 => 24,
            7 => 32,
            _ => {
                return Err(AudioError::InvalidData(
                    "Unexpected bit depth code".to_string(),
                ))
            }
        } as u8;

        // Read and discard 1 reserved bit
        stream.read_bits(1)?;

        let sample_number_or_frame_number = if block_strategy {
            stream.read_bits(36)? as u64 // Read the sample number
        } else {
            stream.read_bits(31)? as u64 // Read the frame number
        };

        // Read the CRC
        let crc = stream.read_bits(8)? as u16;

        // println!("sync_code: {:?}", sync_code);
        // println!("block_size: {:?}", block_size);
        // println!("sample_rate: {}", sample_rate);
        // println!("channel_assignment: {}", channel_assignment);
        // println!("bit_depth: {}", bit_depth);
        // println!(
        //     "sample_number_or_frame_number: {}",
        //     sample_number_or_frame_number
        // );
        // println!("crc: {}", crc);

        // Read subframes
        let mut subframes = Vec::new();
        for _ in 0..channel_assignment {
            let subframe = Subframe::read_next_subframe(stream, bit_depth, block_size as usize)?;
            subframes.push(subframe);
        }

        Ok(Frame {
            sync_code,
            block_strategy,
            block_size,
            sample_rate,
            channel_assignment,
            bit_depth,
            sample_number_or_frame_number,
            crc,
            subframes,
        })
    }

    pub fn get_subframes(&self) -> &Vec<Subframe> {
        &self.subframes
    }
}
