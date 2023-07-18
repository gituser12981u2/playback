// src/codecs/flac/decoder.rs
use crate::common::{errors::AudioError, stream::Stream};

pub struct Decoder {}

impl Decoder {
    pub fn new(_stream: &mut Stream) -> Result<Self, AudioError> {
        Ok(Self {})
    }

    pub fn decode(&mut self, stream: &mut Stream) -> Result<Vec<u8>, AudioError> {
        Ok(vec![])
    }
}
