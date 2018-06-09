use std::io;
use std::io::{Error, ErrorKind};
use std::net::{UdpSocket, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::thread::{sleep, yield_now};
use std::time::{Duration, Instant};

const READING: usize = 0b0001;
const READ: usize    = 0b0010;
const WRITING: usize = 0b0100;
const WRITE: usize   = 0b1000;

pub struct DanSocket {
    socket: UdpSocket,
    connection_address: SocketAddr,
    read_buffer: (SyncSender<Vec<u8>>, Receiver<Vec<u8>>),
    write_buffer: (SyncSender<Vec<u8>>, Receiver<Vec<u8>>),
    flags: AtomicUsize,
    pub(super) packet_size: usize,
    packet_time: Duration,
}

impl DanSocket {

    pub fn create(
        binding_address: Option<SocketAddr>,
        connection_address: SocketAddr,
        read_size: usize,
        write_size: usize,
        packet_size: usize,
        packet_time: Duration,
    ) -> io::Result<DanSocket> {

        // A wildcard address is an address the OS determines
        let wildcard_address = "0.0.0.0:0".parse().unwrap();
        let socket = UdpSocket::bind(binding_address.unwrap_or(wildcard_address))?;

        Ok(DanSocket {
            socket,
            connection_address,
            read_buffer: sync_channel(read_size),
            write_buffer: sync_channel(write_size),
            flags: AtomicUsize::new(READ | WRITE),
            packet_size,
            packet_time,
        })
    }

    pub fn discover_ip(&self, timeout: Duration, packet: &mut [u8]) -> io::Result<()> {
        self.socket.send_to(packet, self.connection_address)?;
        self.socket.set_read_timeout(Some(timeout))?;
        self.socket.recv_from(packet)?;
        self.socket.set_read_timeout(None)?;
        Ok(())
    }

    pub fn read(&self) -> Option<Vec<u8>> {
        self.read_buffer.1.try_recv().ok()
    }

    pub fn write(&self, packet: Vec<u8>) -> bool {
        self.write_buffer.0.try_send(packet).is_ok()
    }

    pub fn read_all(&self) -> io::Result<()> {
        let first = self.flags.load(Ordering::Relaxed);
        // Check if READ flag was set AND READING was NOT set
        if ((first & READ) != 0) && ((first & READING) == 0) {

            let previous = self.flags.compare_and_swap(first, first | READING, Ordering::Relaxed);
            if previous == first {

                let packet = &mut Vec::with_capacity(self.packet_size);
                while (self.flags.load(Ordering::Relaxed) & READ) != 0 {
                    if self.socket.peek_from(packet).is_ok() { // Will never block
                        let (packet_size, address) = self.socket.recv_from(packet)?;

                        if address != self.connection_address {
                            let error_message = format!("Unexpected Address: {}", address);
                            return Err(Error::new(ErrorKind::AddrNotAvailable, error_message));
                        }

                        if packet_size == self.packet_size {
                            if self.read_buffer.0.try_send(packet.clone()).is_err() {
                                self.read(); // Pops from head of buffer to make room
                                self.read_buffer.0.try_send(packet.clone()).unwrap();
                            }
                        }
                    }
                }

                self.flags.fetch_and(!READING, Ordering::Relaxed);
                return Ok(());
            }

            // Attempt to read again
            return self.read_all();
        }

        Err(Error::new(ErrorKind::Other, "Reading is inaccessible for this thread!"))
    }

    pub fn write_all(&self) -> io::Result<()> {
        let first = self.flags.load(Ordering::Relaxed);
        // Check if WRITE flag was set AND WRITING was NOT set
        if ((first & WRITE) != 0) && ((first & WRITING) == 0) {

            let previous = self.flags.compare_and_swap(first, first | WRITING, Ordering::Relaxed);
            if previous == first {

                let mut time = Instant::now();
                let default = Duration::new(0, 0);
                let packet_time = self.packet_time;
                let connection_address = self.connection_address;
                while (self.flags.load(Ordering::Relaxed) & WRITE) != 0 {
                    let packet = self.write_buffer.1.try_recv();
                    if packet.is_ok() {
                        let packet = packet.unwrap();
                        let packet = packet.as_slice();

                        sleep(packet_time.checked_sub(Instant::now() - time).unwrap_or(default));
                        self.socket.send_to(packet, connection_address)?;
                        time = Instant::now();
                    }
                }

                self.flags.fetch_and(!WRITING, Ordering::Relaxed);
                return Ok(());
            }

            // Attempt to write again
            return self.write_all();
        }

        Err(Error::new(ErrorKind::Other, "Writing is inaccessible for this thread!"))
    }
}

impl Drop for DanSocket {

    fn drop(&mut self) {
        self.flags.fetch_and(!(READ | WRITE), Ordering::Relaxed);
        while (self.flags.load(Ordering::Relaxed) & (READING | WRITING)) != 0 {
            yield_now(); // Waits until both READING and WRITING threads finish
        }
    }
}
