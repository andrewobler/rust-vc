#![warn(clippy::pedantic)]

mod audio;
mod constants;
mod errors;
mod sample_ring_buffer;
mod udp;
mod util;

use std::io;

use cpal::traits::StreamTrait;
use simple_logger::SimpleLogger;

use crate::{constants::SAMPLE_RING_BUF_SIZE, sample_ring_buffer::Buffer, udp::AudioSocket};

fn main() {
    SimpleLogger::new().init().unwrap();

    let input_ring_buf = Buffer::new_handle(SAMPLE_RING_BUF_SIZE);
    let output_ring_buf = Buffer::new_handle(SAMPLE_RING_BUF_SIZE);

    let input_stream = audio::create_default_input_stream(input_ring_buf.clone()).unwrap();
    let output_stream = audio::create_default_output_stream(output_ring_buf.clone()).unwrap();

    let send_socket = AudioSocket::bind("0.0.0.0:4444").unwrap();
    let recv_socket = AudioSocket::bind("0.0.0.0:5555").unwrap();

    send_socket.connect("127.0.0.1:5555").unwrap();
    recv_socket.connect("127.0.0.1:4444").unwrap();

    send_socket.spawn_send_thread(input_ring_buf.clone());
    recv_socket.spawn_recv_thread(output_ring_buf.clone());

    input_stream.play().unwrap();
    output_stream.play().unwrap();

    let stdin = io::stdin();

    println!("Press ENTER/RETURN to end...");

    let mut buf = String::new();
    let _ = stdin.read_line(&mut buf);

    input_stream.pause().unwrap();
    output_stream.pause().unwrap();
}
