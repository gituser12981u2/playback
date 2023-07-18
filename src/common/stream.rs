// src/common/stream.rs
use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

use super::errors::AudioError;

pub struct Stream {
    reader: BufReader<File>,
}

impl Stream {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, AudioError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self { reader })
    }

    pub fn reader(&mut self) -> &mut BufReader<File> {
        &mut self.reader
    }

    // reads one byte from the stream
    pub fn read_byte(&mut self) -> Result<u8, AudioError> {
        let mut buffer = [0; 1];
        self.reader.read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    // reads specified number of bytes from the stream
    pub fn read_bytes(&mut self, num: usize) -> Result<Vec<u8>, AudioError> {
        let mut buffer = vec![0; num];
        self.reader.read_exact(&mut buffer).map_err(|err| {
            if err.kind() == io::ErrorKind::UnexpectedEof {
                AudioError::EOF
            } else {
                AudioError::from(err)
            }
        })?;
        Ok(buffer)
    }
}
