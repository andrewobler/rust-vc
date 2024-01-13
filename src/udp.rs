use std::{
    io,
    net::UdpSocket,
    sync::{mpsc, Arc},
    thread::{self, JoinHandle},
};

use log::{error, warn};

// TODO: definitely change this
const RECV_BUF_SIZE: usize = 65536;

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

    pub fn spawn_send_thread(&self, audio_rx: mpsc::Receiver<Vec<u8>>) -> JoinHandle<()> {
        let socket = Arc::clone(&self.socket);

        thread::spawn(move || {
            loop {
                let Ok(packet) = audio_rx.recv() else {
                    error!("Audio RX disconnected, stopping send loop");
                    break;
                };

                // TODO: handle partial sends?
                // TODO: split into RECV_BUF_SIZE chunks if necessary
                if let Err(e) = socket.send(packet.as_slice()) {
                    error!("Failed to send packet of size {}: {}", packet.len(), e);
                }
            }

            warn!("Send thread has stopped");
        })
    }

    pub fn spawn_recv_thread(&self, audio_tx: mpsc::Sender<Vec<u8>>) -> JoinHandle<()> {
        let socket = Arc::clone(&self.socket);

        thread::spawn(move || {
            let mut buf: Vec<u8> = vec![cpal::Sample::EQUILIBRIUM; RECV_BUF_SIZE];

            loop {
                match socket.recv(buf.as_mut_slice()) {
                    Ok(num_samples) => {
                        let packet = buf[..num_samples].to_vec();
                        // TODO: revise to hit this case outside of socket.recv()
                        if audio_tx.send(packet).is_err() {
                            error!("Audio TX disconnected, stopping recv loop");
                            break;
                        }
                    }
                    Err(e) => error!("Failed to receive packet: {e}"),
                }
            }

            warn!("Recv thread has stopped");
        })
    }
}
