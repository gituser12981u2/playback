// src/common/errors.rs
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Unsupported file codec")]
    UnsupportedFileCodec,

    // #[error("{0}")]
    // Custom(String),
    #[error("Unexpected end of file")]
    EOF,
}

impl AudioError {
    // pub fn new(msg: String) -> Self {
    //     AudioError::Custom(msg)
    // }

    pub fn is_eof(&self) -> bool {
        matches!(self, AudioError::EOF)
    }
}
