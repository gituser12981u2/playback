// src/common/pcm.rs
use super::{errors::AudioError, stream::Stream};
use crate::codecs::flac::flac::FLAC;
use std::{io::Read, str};

pub struct PCM {
    // This should be your PCM data structure, Vec<u8> is a placeholder here
    data: Vec<u8>,
}

impl PCM {
    pub fn from_stream(stream: &mut Stream) -> Result<Self, AudioError> {
        println!("Starting from_stream method...");

        // determine the file type
        let mut buffer = [0; 4]; // find the magic bytes
        stream.reader().read_exact(&mut buffer)?;

        let data = match str::from_utf8(&buffer) {
            Ok("fLaC") => {
                let mut flac = FLAC::new(stream)?;

                let metadata = flac.get_metadata();
                println!("{:?}", metadata.get_short_format());
                println!("{:?}", metadata.get_long_format());

                let data = flac.decode(stream)?;
                data
            }
            // Ok("ID3\x03") => {
            //     println!("mp3 file");
            //     let mut mp3 = Mp3::new(stream)?;
            //     mp3.decode()
            // }
            // Ok("OggS") => {
            //     println!("ogg vorbis file");
            //     let mut ogg = OGG::new(stream)?;
            //     ogg.decode()
            // }
            _ => return Err(AudioError::UnsupportedFileCodec),
        };

        Ok(Self { data })
    }

    pub fn play(&self) -> Result<(), AudioError> {
        // send PCM data to the audio hardware
        unimplemented!()
    }
}
