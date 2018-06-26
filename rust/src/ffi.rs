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
use net::DanSocket;
use std::ffi::CStr;
use std::os::raw::{c_char, c_uchar, c_ulonglong};
use std::ptr::null_mut;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::time::Duration;

#[no_mangle]
pub unsafe extern "C" fn dan_create(
    binding_address: *const c_char,
    connection_address: *const c_char,
    socket_timeout: c_ulonglong,
) -> *mut DanSocket {

    let connection_address = CStr::from_ptr(connection_address);
    let binding_address = match binding_address.is_null() {
        false => Some(CStr::from_ptr(binding_address)),
        true => None
    };

    let binding_address = binding_address.map(|cstr| cstr.to_str().unwrap().parse());
    let connection_address = connection_address.to_str().unwrap().parse();

    // Parsing socket address failed
    if connection_address.is_err() {
        return null_mut();
    }

    // binding_address exists / was provided, but parsing socket address failed
    if binding_address.is_some() && binding_address.clone().unwrap().is_err() {
        return null_mut();
    }

    let socket_timeout = if socket_timeout != 0 {
        Some(Duration::from_nanos(socket_timeout))
    } else {
        None
    };

    let binding_address = binding_address.map(|binding_address| binding_address.unwrap());
    let connection_address = connection_address.unwrap();
    let dan = DanSocket::create(binding_address, connection_address, socket_timeout);
    dan.map(|dan| Box::into_raw(Box::new(dan))).unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn dan_destroy(dan: *mut DanSocket) {
    let dan = Box::from_raw(dan); // Box frees when dropped
    dan.destroy()
}

#[no_mangle]
pub unsafe extern "C" fn dan_discover_ip(
    dan: *const DanSocket,
    packet: *mut c_uchar,
    packet_size: usize,
) -> bool {

    let buffer_packet = &mut Vec::with_capacity(packet_size);
    (&*dan).discover_ip(buffer_packet).map(|_| {
        let packet = from_raw_parts_mut(packet, packet_size);
        packet.clone_from_slice(buffer_packet)
    }).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn dan_reading(dan: *const DanSocket, packet_size: usize) -> bool {
    // Function may block until dan_destroy is invoked or an error reading occurs
    (&*dan).read_socket.read(packet_size).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn dan_read(
    dan: *const DanSocket,
    packet: *mut c_uchar,
    packet_size: usize,
) -> bool {

    (&*dan).read_buffer.try_recv().map(|buffer_packet| {
        let packet = from_raw_parts_mut(packet, packet_size);
        packet.clone_from_slice(&buffer_packet)
    }).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn dan_received(dan: *const DanSocket) -> usize {
    (&*dan).received()
}

#[no_mangle]
pub unsafe extern "C" fn dan_writing(dan: *const DanSocket, packet_time: c_ulonglong) -> bool {
    // Function may block until dan_destroy is invoked or an error writing occurs
    (&*dan).write_socket.write(Duration::from_nanos(packet_time)).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn dan_write(
    dan: *const DanSocket,
    packet: *const c_uchar,
    packet_size: usize,
) -> bool {

    let packet = from_raw_parts(packet, packet_size);
    (&*dan).write_buffer.send(packet.to_vec()).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn dan_sent(dan: *const DanSocket) -> usize {
    (&*dan).sent()
}
