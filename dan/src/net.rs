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
use std::net::{UdpSocket, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::thread::{sleep, yield_now};
use std::time::{Duration, Instant};

const READ: usize    = 0b0001;
const READING: usize = 0b0010;
const WRITE: usize   = 0b0100;
const WRITING: usize = 0b1000;

pub struct DanSocket {
    socket: UdpSocket,
    connection_address: SocketAddr,
    flags: AtomicUsize,
    pub(crate) packet_size: usize,
    packet_time: Duration,
}

pub struct DanReader(Receiver<Vec<u8>>);

pub struct DanReadSocket {
    socket: Arc<DanSocket>,
    buffer: SyncSender<Vec<u8>>,
}

pub struct DanWriter {
    socket : Arc<DanSocket>,
    buffer: SyncSender<Vec<u8>>,
}

pub struct DanWriteSocket {
    socket: Arc<DanSocket>,
    buffer: Receiver<Vec<u8>>,
}

pub struct Dan {
    pub socket: Arc<DanSocket>,
    pub reader: DanReader,
    pub read_socket: DanReadSocket,
    pub writer: DanWriter,
    pub write_socket: DanWriteSocket,
}

impl Dan {

    pub fn create(
        binding_address: Option<SocketAddr>,
        connection_address: SocketAddr,
        socket_timeout: Option<Duration>,
        read_size: usize,
        write_size: usize,
        packet_size: usize,
        packet_time: Duration,
    ) -> io::Result<Dan> {

        // A wildcard address is an address the OS determines
        let wildcard_address = "0.0.0.0:0".parse().unwrap();
        let socket = UdpSocket::bind(binding_address.unwrap_or(wildcard_address))?;
        socket.set_read_timeout(Some(socket_timeout.unwrap_or(Duration::from_secs(1))))?;

        let flags = AtomicUsize::new(READ | WRITE);
        let socket = DanSocket { socket, connection_address, flags, packet_time, packet_size };
        let socket = Arc::new(socket);

        let (sender, receiver) = sync_channel(read_size);
        let reader = DanReader(receiver);
        let read_socket = DanReadSocket { socket: socket.clone(), buffer: sender };

        let (sender, receiver) = sync_channel(write_size);
        let writer = DanWriter { socket: socket.clone(), buffer: sender };
        let write_socket = DanWriteSocket { socket: socket.clone(), buffer: receiver };

        Ok(Dan { socket, reader, read_socket, writer, write_socket })
    }

    pub fn destroy(dan: &DanSocket) {
        while (dan.flags.fetch_and(!(READ | WRITE), Ordering::Relaxed) & (READING | WRITING)) != 0 {
            yield_now(); // Yields until READING/WRITING flags are disabled by disabling READ/WRITE
        }
    }

    pub fn discover_ip(&self, packet: &mut Vec<u8>) -> io::Result<()> {
        self.socket.socket.send_to(packet, self.socket.connection_address)?;
        self.socket.socket.recv_from(packet)?;
        Ok(())
    }

    pub fn read(dan: &DanReader) -> Option<Vec<u8>> {
        dan.0.try_recv().ok()
    }

    pub fn read_socket(dan: &DanReadSocket) -> io::Result<()> {
        let connection_address = dan.socket.connection_address;
        let packet_size = dan.socket.packet_size;
        let packet = &mut Vec::with_capacity(packet_size);
        let socket = &dan.socket.socket;
        let flags = &dan.socket.flags;
        let buffer = &dan.buffer;

        // Flag a thread is READING, and process until no longer to READ
        while (flags.fetch_or(READING, Ordering::Relaxed) & READ) != 0 {
            let received = socket.recv_from(packet);
            if received.is_ok() {

                let (size, address) = received.unwrap();
                if (size != packet_size) || (address != connection_address) {
                    flags.fetch_and(!READING, Ordering::Relaxed); // Breaking out of the whole loop!
                    let error_message = format!("Unexpected Packet Size {} from {}", size, address);
                    return Err(Error::new(ErrorKind::Other, error_message));
                }

                buffer.try_send(packet.clone()).ok();
            }
        }

        flags.fetch_and(!READING, Ordering::Relaxed);
        Ok(())
    }

    pub fn write(dan: &DanWriter, packet: Vec<u8>) -> bool {
        if dan.socket.packet_size == packet.len() {
            return dan.buffer.try_send(packet).is_ok()
        }

        false
    }

    pub fn write_socket(dan: &DanWriteSocket) -> io::Result<()> {
        let connection_address = dan.socket.connection_address;
        let packet_time = dan.socket.packet_time;
        let socket = &dan.socket.socket;
        let flags = &dan.socket.flags;
        let buffer = &dan.buffer;
        let default = Duration::new(0, 0);
        let mut time = Instant::now();

        // Flag a thread is WRITING, and process until no longer to WRITE
        while (flags.fetch_or(WRITING, Ordering::Relaxed) & WRITE) != 0 {

            let packet = buffer.try_recv();
            if packet.is_ok() {
                let packet = packet.unwrap();

                sleep(packet_time.checked_sub(Instant::now() - time).unwrap_or(default));
                let sent = socket.send_to(&packet, connection_address);
                time = Instant::now();

                if sent.is_err() { // Breaking out of while loop!
                    flags.fetch_and(!WRITING, Ordering::Relaxed);
                    return sent.map(|_| ());
                }
            }
        }

        flags.fetch_and(!WRITING, Ordering::Relaxed);
        Ok(())
    }
}
