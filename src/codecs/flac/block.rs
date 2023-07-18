// src/codesc/flac/block.rs
use crate::common::{errors::AudioError, stream::Stream};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Block {
    block_type: BlockType,
    length: usize,
    data: Vec<u8>,
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

// read and return the next block from the stream
pub fn read_next_block(stream: &mut Stream) -> Result<Option<Block>, AudioError> {
    // read first byte from the stream
    let first_byte = match stream.read_byte() {
        Ok(byte) => byte,
        Err(ref e) if e.is_eof() => return Ok(None),
        Err(e) => return Err(e),
    };

    // determine whether its the last metadata block
    let is_last = first_byte & 0x80 != 0;

    // get block type from the 7 lower bits
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

    // read next 3 bytes to get the block length
    let block_length_bytes = match stream.read_bytes(3) {
        Ok(bytes) => bytes,
        Err(ref e) if e.is_eof() => return Ok(None),
        Err(e) => return Err(e),
    };

    let length = ((block_length_bytes[0] as usize) << 16)
        | ((block_length_bytes[1] as usize) << 8)
        | block_length_bytes[2] as usize;

    // read the data in the block
    let data = match stream.read_bytes(length) {
        Ok(bytes) => bytes,
        Err(ref e) if e.is_eof() => return Ok(None),
        Err(e) => return Err(e),
    };

    // handle the error if its the last block
    if is_last {
        return Ok(None);
    }

    Ok(Some(Block {
        block_type,
        length,
        data,
    }))
}
