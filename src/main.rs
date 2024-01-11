#![warn(clippy::pedantic)]

mod audio;
mod errors;

use std::{io, sync::mpsc};

use cpal::traits::StreamTrait;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new().init().unwrap();
    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    let input_stream = audio::create_default_input_stream(tx).unwrap();
    let output_stream = audio::create_default_output_stream(rx).unwrap();

    input_stream.play().unwrap();
    output_stream.play().unwrap();

    let stdin = io::stdin();

    println!("Press ENTER/RETURN to end...");

    let mut buf = String::new();
    let _ = stdin.read_line(&mut buf);

    input_stream.pause().unwrap();
    output_stream.pause().unwrap();
}
