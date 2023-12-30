// src/common/errors.rs
use std::io;
use thiserror::Error;

/**  
 * Define the AudioError enum which represents possible errors that may occur
 * in the app
*/
#[derive(Debug, Error)]
pub enum AudioError {
    // Represents an I/O error, carries the underlying io::Error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    // Represents an error when an unsupported file codec is used
    #[error("Unsupported file codec")]
    UnsupportedFileCodec,

    // Represents a parsing error, carries the error message as a string
    #[error("Parse error: {0}")]
    ParseError(String),

    // Represents an error when an unexpected end of file (EOF) is reached
    #[error("Unexpected end of file")]
    EOF,

    // Represents an error when invalid data is encountered
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Attempted to read more than 32 bits")]
    ExceededBitLimit,

    #[error("ArithmeticOverflow")]
    ArithmeticOverflow,
}

// Implementation of helper methods for the AudioError enum
impl AudioError {
    // Method to check if the error is due to reaching the end of the file
    pub fn is_eof(&self) -> bool {
        matches!(self, AudioError::EOF)
    }
}
