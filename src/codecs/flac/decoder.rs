// src/codecs/flac/decoder.rs
use crate::common::{errors::AudioError, stream::Stream};

use super::{
    block::{read_next_block, METADATA_BLOCK_TYPES},
    frame::Frame,
};

pub struct Decoder {}

impl Decoder {
    pub fn new(_stream: &mut Stream) -> Result<Self, AudioError> {
        Ok(Self {})
    }

    pub fn decode(&mut self, stream: &mut Stream) -> Result<Vec<u32>, AudioError> {
        let mut pcm_data = vec![];
        let mut is_last_metadata_block = false;

        while !is_last_metadata_block {
            // Read next metadata block
            match read_next_block(stream, &METADATA_BLOCK_TYPES) {
                Ok(Some(block)) => {
                    // println!("metadata block: {:?}", block);
                    // println!("block type: {:?}", block.get_type());
                    // println!("block length: {:?}", block.get_length());
                }
                Ok(None) => {
                    is_last_metadata_block = true;
                }
                Err(err) => return Err(err),
            }
        }

        loop {
            let header_result = stream.peek_u16();
            match header_result {
                Ok(header) => {
                    if header == 0xFFF8 {
                        println!("Found frame header");
                        // Frame sync code found
                        match Frame::read_next_frame(stream) {
                            Ok(frame) => {
                                println!("decoding frame");
                                for subframe in frame.get_subframes() {
                                    pcm_data.extend_from_slice(&subframe.get_data());
                                }
                            }
                            Err(err) => return Err(err),
                        }
                    } else {
                        return Err(AudioError::InvalidData(
                            "Expected frame header not found".to_string(),
                        ));
                    }
                }
                Err(err) => {
                    if let AudioError::EOF = err {
                        break; // If EOF is reached, exit the loop
                    }
                    return Err(err); // Return error if it's anything else
                }
            }
        }
        Ok(pcm_data)
    }
}
