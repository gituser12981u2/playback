// src/common/stream.rs
use std::{
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use byteorder::{BigEndian, ReadBytesExt};

use super::errors::AudioError;

// Define a Stream struct which holds a BufReader to a File
pub struct Stream {
    reader: BufReader<File>,
}

impl Stream {
    /**  
     * The constructor method takes a reference to a Path and returns a
     * Result that may contain a Stream or an AudioError
     */
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, AudioError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self { reader })
    }

    // Method to access the mutable reference to the BufReader
    pub fn reader(&mut self) -> &mut BufReader<File> {
        &mut self.reader
    }

    // Method to read a specified number of bits from the stream
    pub fn read_bits(&mut self, num: u8) -> Result<u32, AudioError> {
        if num > 32 {
            return Err(AudioError::ExceededBitLimit);
        }

        let mut value = 0;
        for _ in 0..num {
            let bit = self.read_bit()?;
            value = (value << 1) | bit;
        }
        Ok(value)
    }

    // Method to read a single bit from the stream
    pub fn read_bit(&mut self) -> Result<u32, AudioError> {
        static mut BIT_POS: u8 = 0;
        static mut CACHE: u8 = 0;
        unsafe {
            if BIT_POS == 0 {
                CACHE = self.read_byte()?;
                BIT_POS = 8;
            }
            BIT_POS -= 1;
            Ok(((CACHE >> BIT_POS) & 1) as u32)
        }
    }

    // Method to read a specified number of bits from the stream and interpret them as a signed integer
    pub fn read_signed_bits(&mut self, num: u8) -> Result<i32, AudioError> {
        if num > 32 {
            return Err(AudioError::ExceededBitLimit);
        }

        let mut value = self.read_bits(num)? as i32; // Read the bits as an unsigned integer

        // If the number is negative (most significant bit is 1), adjust the value
        if value & (1 << (num - 1)) != 0 {
            value -= 1 << num; // Subtract 2^num to get the correct negative value
        }
        Ok(value)
    }

    /**
     * Metod to read one byte from the stream
     * Returns a Result that may contain a u8 or an AudioError
     */
    pub fn read_byte(&mut self) -> Result<u8, AudioError> {
        let mut buffer = [0; 1];
        self.reader.read_exact(&mut buffer).map_err(|err| {
            if err.kind() == io::ErrorKind::UnexpectedEof {
                AudioError::EOF
            } else {
                AudioError::from(err)
            }
        })?;
        Ok(buffer[0])
    }

    /**
     * Method to read a specified number of bytes from the stream
     * Returns a Result that may contain a Vec<u8> or an AudioError
     */
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

    // Method to read a 16-bit unsigned integer from the stream
    pub fn read_u16(&mut self) -> Result<u16, AudioError> {
        self.reader
            .read_u16::<BigEndian>()
            .map_err(AudioError::from)
    }

    // Method to read a 32-bit unsigned integer from the stream
    pub fn read_u32(&mut self) -> Result<u32, AudioError> {
        self.reader
            .read_u32::<BigEndian>()
            .map_err(AudioError::from)
    }

    // Method to read a 64-bit unsigned integer from the stream
    pub fn read_u64(&mut self) -> Result<u64, AudioError> {
        self.reader
            .read_u64::<BigEndian>()
            .map_err(AudioError::from)
    }

    pub fn peek_u16(&mut self) -> Result<u16, AudioError> {
        let original_position = self.reader.stream_position()?;
        let result = self.read_u16();
        self.reader.seek(SeekFrom::Start(original_position))?;
        result
    }

    pub fn peek_u32(&mut self) -> Result<u32, AudioError> {
        let original_position = self.reader.stream_position()?;
        let result = self.read_u32();
        self.reader.seek(SeekFrom::Start(original_position))?;
        result
    }

    // Method to skip a certain number of bytes in the stream
    pub fn skip(&mut self, num_bytes: usize) -> Result<(), AudioError> {
        self.reader
            .seek(SeekFrom::Current(num_bytes as i64))
            .map_err(AudioError::from)
            .map(|_| ())
    }
}
