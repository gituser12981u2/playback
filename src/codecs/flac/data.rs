// src/codecs/flac/data.rs

// This module handles parsing FLAC metadata from the input stream.

use super::block::{read_next_block, Block, BlockType};
use crate::common::{errors::AudioError, stream::Stream};
use byteorder::{BigEndian, ReadBytesExt};
use std::sync::Arc;

// Struct to hold parsed metadata of FLAC file
pub struct Metadata {
    min_block_size: Option<u16>,
    max_block_size: Option<u16>,
    min_frame_size: Option<u32>,
    max_frame_size: Option<u32>,
    sample_rate: Option<u32>,
    num_channels: Option<u8>,
    bit_depth: Option<u8>,
    total_samples: Option<u64>,
    md5_signature: Option<Arc<Vec<u8>>>, // Arc is used for efficient shared ownership
}

impl Metadata {
    // Constructor that reads and parses metadata from a given Stream
    pub fn new(stream: &mut Stream) -> Result<Self, AudioError> {
        // Initialization of Metadata with None values
        let mut metadata = Metadata {
            min_block_size: None,
            max_block_size: None,
            min_frame_size: None,
            max_frame_size: None,
            sample_rate: None,
            num_channels: None,
            bit_depth: None,
            total_samples: None,
            md5_signature: None,
        };

        // Specify which block types to read
        let read_types = &[BlockType::StreamInfo, BlockType::VorbisComment];

        // Loop to read blocks of the specified types
        while let Some(block) = read_next_block(stream, read_types)? {
            // Use block type to determine how to parse it
            match block.get_type() {
                BlockType::StreamInfo => metadata.parse_stream_info(block)?,
                BlockType::VorbisComment => metadata.parse_vorbis_comment(block)?,
                // Should not reach here as we only read the specified block types
                _ => {
                    return Err(AudioError::ParseError(
                        "Unexpected block type encountered".into(),
                    ))
                }
            }
        }

        Ok(metadata)
    }

    // Parse STREAMINFO block and poplate corresponding fields in Metadata
    fn parse_stream_info(&mut self, block: Block) -> Result<(), AudioError> {
        let data = block.get_data();

        // Check if the block has enough data to parse STREAMINFO block
        if data.len() < 34 {
            return Err(AudioError::ParseError(
                "Not enough data to parse STREAMINFO block".into(),
            ));
        }

        // Parse binary data according to flac specs
        let min_block_size = Some((&data[0..2]).read_u16::<BigEndian>()?);
        let max_block_size = Some((&data[2..4]).read_u16::<BigEndian>()?);
        let min_frame_size = Some((&data[4..7]).read_u24::<BigEndian>()?);
        let max_frame_size = Some((&data[7..10]).read_u24::<BigEndian>()?);

        /*
         * The sample rate (in Hz) is a 20-but integer
         * It's stored in the upper 20 bits of a 32-bit block
         */
        let sample_rate_raw = (&data[10..14]).read_u32::<BigEndian>()?;
        let sample_rate = Some(sample_rate_raw >> 12);

        /*
         * The number of channels is a 3-bit number
         * It's stored in the 2nd, 3rd, and 4th bits of the 13th byte
         */
        let num_channels = Some((((data[12] & 0x0E) >> 1) + 1) as u8);

        /*
         * The bit deptth per sample is a 5-bit number
         * It's stored in the last bit of the 13th byte and the first 4 bits of the 14th byte
         */
        let bit_depth = Some((((data[12] & 0x01) << 4) | (data[13] >> 4) + 1) as u8);

        /*
         * The total number of samples is a 36-bit number
         * It's stored in the last 4 bites of the 14th byte and the 15th to 18th bytes
         */
        let total_samples = Some(
            ((data[13] as u64 & 0x0F) << 32) | ((&data[14..18]).read_u32::<BigEndian>()? as u64),
        );

        let md5_signature = Some(Arc::new(data[18..34].to_vec()));

        // Assign extracted values to Metadata fields
        self.min_block_size = min_block_size;
        self.max_block_size = max_block_size;
        self.min_frame_size = min_frame_size;
        self.max_frame_size = max_frame_size;
        self.sample_rate = sample_rate;
        self.num_channels = num_channels;
        self.bit_depth = bit_depth;
        self.total_samples = total_samples;
        self.md5_signature = md5_signature;

        Ok(())
    }

    // Parse VORBIS_COMMENT block and poplate corresponding fields in Metadata
    fn parse_vorbis_comment(&mut self, block: Block) -> Result<(), AudioError> {
        let _data = block.get_data();

        // TODO: Parse binary data according to flac specs

        Ok(())
    }

    pub fn get_min_block_size(&self) -> Option<u16> {
        self.min_block_size
    }

    pub fn get_max_block_size(&self) -> Option<u16> {
        self.max_block_size
    }

    pub fn get_min_frame_size(&self) -> Option<u32> {
        self.min_frame_size
    }

    pub fn get_max_frame_size(&self) -> Option<u32> {
        self.max_frame_size
    }

    pub fn get_sample_rate(&self) -> Option<u32> {
        self.sample_rate
    }

    pub fn get_num_channels(&self) -> Option<u8> {
        self.num_channels
    }

    pub fn get_bit_depth(&self) -> Option<u8> {
        self.bit_depth
    }

    pub fn get_total_samples(&self) -> Option<u64> {
        self.total_samples
    }

    pub fn get_md5_signature(&self) -> Option<Arc<Vec<u8>>> {
        self.md5_signature.clone()
    }

    /**
     * These two methods return a ShortFormat of LongFormat struct, which
     * are simplified version of the Metadata for use in other parts of the code
     */
    pub fn get_short_format(&self) -> ShortFormat {
        ShortFormat {
            sample_rate: self.get_sample_rate(),
            num_channels: self.get_num_channels(),
            bit_depth: self.get_bit_depth(),
        }
    }

    pub fn get_long_format(&self) -> LongFormat {
        LongFormat {
            min_block_size: self.get_min_block_size(),
            max_block_size: self.get_max_block_size(),
            min_frame_size: self.get_min_frame_size(),
            max_frame_size: self.get_max_frame_size(),
            sample_rate: self.get_sample_rate(),
            num_channels: self.get_num_channels(),
            bit_depth: self.get_bit_depth(),
            total_samples: self.get_total_samples(),
            md5_signature: self.get_md5_signature(),
        }
    }
}

#[derive(Debug)]
pub struct ShortFormat {
    pub sample_rate: Option<u32>,
    pub num_channels: Option<u8>,
    pub bit_depth: Option<u8>,
}

#[derive(Debug)]
pub struct LongFormat {
    pub min_block_size: Option<u16>,
    pub max_block_size: Option<u16>,
    pub min_frame_size: Option<u32>,
    pub max_frame_size: Option<u32>,
    pub sample_rate: Option<u32>,
    pub num_channels: Option<u8>,
    pub bit_depth: Option<u8>,
    pub total_samples: Option<u64>,
    pub md5_signature: Option<Arc<Vec<u8>>>,
}
