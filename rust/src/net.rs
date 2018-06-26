// This file is part of DAN.
//
// DAN is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// DAN is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with DAN.  If not, see <http://www.gnu.org/licenses/>.
use std::io;
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering::Relaxed};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{sleep, yield_now};
use std::time::{Duration, Instant};

const READ: usize    = 0b0001;
const READING: usize = 0b0010;
const WRITE: usize   = 0b0100;
const WRITING: usize = 0b1000;

pub struct DanSharedSocket {
    socket: UdpSocket,
    address: SocketAddr,
    flags: AtomicUsize,
    received: AtomicUsize,
    sent: AtomicUsize,
}

pub struct DanReadSocket {
    shared_socket: Arc<DanSharedSocket>,
    read_buffer: Sender<Vec<u8>>,
}

pub struct DanWriteSocket {
    shared_socket: Arc<DanSharedSocket>,
    write_buffer: Receiver<Vec<u8>>,
}

pub struct DanSocket {
    shared_socket: Arc<DanSharedSocket>,
    pub read_socket: DanReadSocket,
    pub read_buffer: Receiver<Vec<u8>>,
    pub write_socket: DanWriteSocket,
    pub write_buffer: Sender<Vec<u8>>,
}

impl DanReadSocket {

    pub fn read(&self, packet_size: usize) -> io::Result<()> {
        let packet = &mut Vec::with_capacity(packet_size);
        let received_address = self.shared_socket.address;
        let socket = &self.shared_socket.socket;
        let flags = &self.shared_socket.flags;
        let buffer = &self.read_buffer;

        // Flag thread is READING, and loop until READ is unset
        while (flags.fetch_or(READING, Relaxed) & READ) != 0 {
            let received = socket.recv_from(packet);
            if received.is_ok() {

                let (size, address) = received.unwrap();
                if (size != packet_size) || (address != received_address) {
                    flags.fetch_and(!READING, Relaxed);
                    let error_message = format!("Unexpected Packet Size {} or Address {}
                    Expected {} and {}", size, address, packet_size, received_address);
                    return Err(Error::new(ErrorKind::Other, error_message));
                }

                buffer.send(packet.clone()).map_err(|error| {
                    flags.fetch_and(!READING, Relaxed);
                    Error::new(ErrorKind::Other, error)
                })?;
                self.shared_socket.received.fetch_add(1, Relaxed);
            }
        }

        flags.fetch_and(!READING, Relaxed);
        Ok(())
    }
}

impl DanWriteSocket {

    pub fn write(&self, packet_time: Duration) -> io::Result<()> {
        let send_address = self.shared_socket.address;
        let socket = &self.shared_socket.socket;
        let flags = &self.shared_socket.flags;
        let buffer = &self.write_buffer;
        let default = Duration::new(0, 0);
        let mut time = Instant::now();

        // Flag thread is WRITING, and loop until WRITE is unset
        while (flags.fetch_or(WRITING, Relaxed) & WRITE) != 0 {

            let packet = buffer.try_recv();
            if packet.is_ok() {
                let packet = packet.unwrap();

                sleep(packet_time.checked_sub(Instant::now() - time).unwrap_or(default));
                socket.send_to(&packet, send_address).map_err(|error| {
                    flags.fetch_and(!WRITING, Relaxed);
                    error
                })?;
                time = Instant::now();
                self.shared_socket.sent.fetch_add(1, Relaxed);
            }
        }

        flags.fetch_and(!WRITING, Relaxed);
        Ok(())
    }
}

impl DanSocket {

    pub fn create(
        binding_address: Option<SocketAddr>,
        connection_address: SocketAddr,
        socket_timeout: Option<Duration>,
    ) -> io::Result<DanSocket> {

        // A wildcard address is an address the OS determines
        let wildcard_address = "0.0.0.0:0".parse().unwrap();
        let socket = UdpSocket::bind(binding_address.unwrap_or(wildcard_address))?;
        socket.set_read_timeout(Some(socket_timeout.unwrap_or(Duration::from_secs(1))))?;

        let socket = Arc::new(DanSharedSocket {
            socket,
            address: connection_address,
            flags: AtomicUsize::new(READ | WRITE),
            received: ATOMIC_USIZE_INIT,
            sent: ATOMIC_USIZE_INIT,
        });

        let (read_buffer, receiver) = channel();
        let read_socket = DanReadSocket { shared_socket: socket.clone(), read_buffer };

        let (sender, write_buffer) = channel();
        let write_socket = DanWriteSocket { shared_socket: socket.clone(), write_buffer };

        Ok(DanSocket {
            shared_socket: socket,
            read_socket,
            read_buffer: receiver,
            write_socket,
            write_buffer: sender,
        })
    }

    pub fn destroy(&self) {
        let flags = &self.shared_socket.flags;
        while (flags.fetch_and(!(READ | WRITE), Relaxed) & (READING | WRITING)) != 0 {
            yield_now(); // Yields until READING/WRITING is unset and unset READ/WRITE
        }
    }

    pub fn discover_ip(&self, packet: &mut Vec<u8>) -> io::Result<()> {
        self.shared_socket.socket.send_to(packet, self.shared_socket.address)?;
        self.shared_socket.socket.recv_from(packet)?;
        Ok(())
    }

    pub fn received(&self) -> usize {
        self.shared_socket.received.load(Relaxed)
    }

    pub fn sent(&self) -> usize {
        self.shared_socket.sent.load(Relaxed)
    }
}
