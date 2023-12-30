// src/codecs/flac/block.rs

// This module handles reading and parsing block of data from the FLAC stream

use crate::common::{errors::AudioError, stream::Stream};
use std::io::{Seek, SeekFrom};

// Enum to represent the type of FLAC block
#[derive(Debug, PartialEq)]
pub enum BlockType {
    StreamInfo,
    Padding,
    Application,
    SeekTable,
    VorbisComment,
    CueSheet,
    Picture,
    Reserved,
    Invalid,
}

// Struct to represent a block of data in a FLAC stream
#[derive(Debug)]
pub struct Block {
    block_type: BlockType,
    length: usize, // This represents the length of the block in bytes
    data: Vec<u8>, // This is the raw data contained in the block
}

impl Block {
    pub fn get_type(&self) -> &BlockType {
        &self.block_type
    }

    pub fn get_length(&self) -> usize {
        self.length
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

pub const METADATA_BLOCK_TYPES: [BlockType; 7] = [
    BlockType::StreamInfo,
    BlockType::Padding,
    BlockType::Application,
    BlockType::SeekTable,
    BlockType::VorbisComment,
    BlockType::CueSheet,
    BlockType::Picture,
];

// Fuction to Read and return the next block from the stream
pub fn read_next_block(
    stream: &mut Stream,
    read_types: &[BlockType],
) -> Result<Option<Block>, AudioError> {
    // Read first byte from the stream
    let first_byte = match stream.read_byte() {
        Ok(byte) => byte,
        Err(ref e) if e.is_eof() => return Ok(None),
        Err(e) => return Err(e),
    };

    // Determine whether its the last metadata block
    let is_last = first_byte & 0x80 != 0;

    // Get block type from the 7 lower bits
    let block_type_byte = first_byte & 0x7F;
    let block_type = match block_type_byte {
        0 => BlockType::StreamInfo,
        1 => BlockType::Padding,
        2 => BlockType::Application,
        3 => BlockType::SeekTable,
        4 => BlockType::VorbisComment,
        5 => BlockType::CueSheet,
        6 => BlockType::Picture,
        127 => BlockType::Invalid,
        _ => BlockType::Reserved,
    };

    // Read next 3 bytes to get the block length
    let block_length_bytes = match stream.read_bytes(3) {
        Ok(bytes) => bytes,
        Err(ref e) if e.is_eof() => return Ok(None),
        Err(e) => return Err(e),
    };

    let length = ((block_length_bytes[0] as usize) << 16)
        | ((block_length_bytes[1] as usize) << 8)
        | block_length_bytes[2] as usize;

    // Check if we should read this block type
    if !read_types.contains(&block_type) {
        // If not, skip the block
        stream.reader().seek(SeekFrom::Current(length as i64))?;
        return Ok(None);
    }

    // Read the data in the block
    let data = match stream.read_bytes(length) {
        Ok(bytes) => bytes,
        Err(ref e) if e.is_eof() => return Ok(None),
        Err(e) => return Err(e),
    };

    // If it's the last block, return None to indicate end of metadata blocks
    if is_last {
        return Ok(None);
    }

    Ok(Some(Block {
        block_type,
        length,
        data,
    }))
}
