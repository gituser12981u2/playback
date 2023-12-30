// src/codecs/flac/flac.rs
use super::data::Metadata;
use super::decoder::Decoder;
use crate::common::errors::AudioError;
use crate::common::stream::Stream;

// Define FLAC structure holding metadata and decoder
pub struct FLAC {
    metadata: Metadata,
    decoder: Decoder,
}

impl FLAC {
    /**
     * Constructor method to create a new FLAC object
     * It initializes metadata and decoder using the provided stream
     * Returns a Result that may contain a FLAC or an AudioError
     */
    pub fn new(stream: &mut Stream) -> Result<Self, AudioError> {
        let metadata = Metadata::new(stream)?;
        let decoder = Decoder::new(stream)?;
        Ok(Self { metadata, decoder })
    }

    // Method to access FLAC metadata
    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    /**
     *  Method to decode the FLAC stream
     *  Returns a Result that may contain a Vec<u8> or an AudioError
     */
    pub fn decode(&mut self, stream: &mut Stream) -> Result<Vec<u32>, AudioError> {
        self.decoder.decode(stream)
    }
}
