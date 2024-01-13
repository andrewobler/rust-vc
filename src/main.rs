#![warn(clippy::pedantic)]

mod audio;
mod errors;
mod udp;

use std::{io, sync::mpsc};

use cpal::traits::StreamTrait;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new().init().unwrap();

    let (input_tx, input_rx) = mpsc::channel::<Vec<u8>>();
    let (output_tx, output_rx) = mpsc::channel::<Vec<u8>>();

    let input_stream = audio::create_default_input_stream(input_tx).unwrap();
    let output_stream = audio::create_default_output_stream(output_rx).unwrap();

    let send_socket = udp::AudioSocket::bind("0.0.0.0:4444").unwrap();
    let recv_socket = udp::AudioSocket::bind("0.0.0.0:5555").unwrap();

    send_socket.connect("127.0.0.1:5555").unwrap();
    recv_socket.connect("127.0.0.1:4444").unwrap();

    send_socket.spawn_send_thread(input_rx);
    recv_socket.spawn_recv_thread(output_tx);

    input_stream.play().unwrap();
    output_stream.play().unwrap();

    let stdin = io::stdin();

    println!("Press ENTER/RETURN to end...");

    let mut buf = String::new();
    let _ = stdin.read_line(&mut buf);

    input_stream.pause().unwrap();
    output_stream.pause().unwrap();
}
