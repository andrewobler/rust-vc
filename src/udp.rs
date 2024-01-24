use std::{
    io,
    net::UdpSocket,
    sync::Arc,
    thread::{self, JoinHandle},
};

use dasp_sample::Sample;

use log::error;

use crate::{constants::UDP_BUF_SIZE, sample_ring_buffer::BufferHandle, util::write_silence};

pub struct AudioSocket {
    socket: Arc<UdpSocket>,
}

impl AudioSocket {
    pub fn bind(addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        Ok(Self {
            socket: Arc::new(socket),
        })
    }

    pub fn connect(&self, addr: &str) -> io::Result<()> {
        self.socket.connect(addr)
    }

    pub fn spawn_send_thread(&self, audio_src: BufferHandle) -> JoinHandle<()> {
        let socket = Arc::clone(&self.socket);

        thread::spawn(move || {
            let mut buf = [Sample::EQUILIBRIUM; UDP_BUF_SIZE];

            loop {
                match audio_src.lock() {
                    Ok(mut locked) => {
                        let num_written = locked.pop_into(buf.iter_mut());

                        if let Err(e) = socket.send(&buf[..num_written]) {
                            error!("Error sending {num_written} samples: {e}");
                        }

                        write_silence(&mut buf[..num_written]);
                    }
                    Err(e) => error!("Unable to lock audio_src: {e}"),
                };
            }
        })
    }

    pub fn spawn_recv_thread(&self, audio_sink: BufferHandle) -> JoinHandle<()> {
        let socket = Arc::clone(&self.socket);

        thread::spawn(move || {
            let mut buf = [Sample::EQUILIBRIUM; UDP_BUF_SIZE];

            loop {
                match socket.recv(&mut buf) {
                    Ok(num_recved) => match audio_sink.lock() {
                        Ok(mut locked) => {
                            locked.push_all(buf[..num_recved].iter());
                            write_silence(&mut buf[..num_recved]);
                        }
                        Err(e) => error!("Unable to lock audio_sink: {e}"),
                    },
                    Err(e) => error!("Failed to receive packet: {e}"),
                }
            }
        })
    }
}
