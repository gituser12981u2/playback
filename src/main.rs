// src/main.rs
mod codecs;
mod common;

use common::{errors::AudioError, pcm::PCM, stream::Stream};

fn main() -> Result<(), AudioError> {
    // Take the path from the command line
    let args: Vec<String> = std::env::args().collect();
    // Check if the filename was provided
    if args.len() != 2 {
        eprintln!("Usage: cargo run <filename>");
        std::process::exit(1);
    }
    let path = &args[1];

    // Create a Stream with the file at the provided path
    let mut stream = Stream::new(path)?;

    /*
     *  Determine the file type and dispatch to the appropriate decoder,
     *  and return a PCM
     */
    let pcm = PCM::from_stream(&mut stream)?;

    // Play the PCM data
    pcm.play()?;

    Ok(())
}
