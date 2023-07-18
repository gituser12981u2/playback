// src/codecs/flac/flac.rs
use super::data::Metadata;
use super::decoder::Decoder;
use crate::common::errors::AudioError;
use crate::common::stream::Stream;

pub struct FLAC {
    metadata: Metadata,
    decoder: Decoder,
}

impl FLAC {
    pub fn new(stream: &mut Stream) -> Result<Self, AudioError> {
        let metadata = Metadata::new(stream)?;
        let decoder = Decoder::new(stream)?;
        Ok(Self { metadata, decoder })
    }

    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn decode(&mut self, stream: &mut Stream) -> Result<Vec<u8>, AudioError> {
        self.decoder.decode(stream)
    }
}
