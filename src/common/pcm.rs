// src/common/pcm.rs
use super::{errors::AudioError, stream::Stream};
use crate::codecs::flac::flac::FLAC;
use std::{io::Read, str};

// Define PCM structure holding audio data
pub struct PCM {
    // This should be the PCM data structure, Vec<u8> is a placeholder here
    data: Vec<u32>,
}

impl PCM {
    /**
     * Costructor method to create a PCM from a Stream
     * It determines the file type and dispatches to the appropriate decoder
     * Returns a Result that may contain a PCM or an AudioError
     */
    pub fn from_stream(stream: &mut Stream) -> Result<Self, AudioError> {
        // Determine the file type
        let mut buffer = [0; 4]; // Find the magic bytes
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

    /**
     *  Method to play the PCM data
     *  Returns a Result that may contaion () or an AudioError
     */
    pub fn play(&self) -> Result<(), AudioError> {
        // Send PCM data to the audio hardware
        // test for now
        for sample in self.data.iter().take(10) {
            println!("{:?}", sample)
        }

        Ok(())
    }
}
