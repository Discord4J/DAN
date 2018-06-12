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
use net::Dan;
use std::ffi::CStr;
use std::os::raw::{c_char, c_uchar, c_uint};
use std::ptr::null_mut;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::time::Duration;

#[no_mangle]
pub unsafe extern "C" fn dan_create(
    binding_address: *const c_char,
    connection_address: *const c_char,
    socket_timeout: c_uint,
    read_size: usize,
    write_size: usize,
    packet_size: usize,
    packet_time: c_uint,
) -> *mut Dan {

    let connection_address = CStr::from_ptr(connection_address);
    let binding_address = match binding_address.is_null() {
        true => None,
        _ => Some(CStr::from_ptr(binding_address))
    };

    let binding_address = binding_address.map(|cstr| cstr.to_str().unwrap().parse());
    let connection_address = connection_address.to_str().unwrap().parse().ok();

    // Parsing socket address failed
    if connection_address.is_none() {
        return null_mut();
    }

    // binding_address exists / was provided, but parsing socket address failed
    if binding_address.is_some() && binding_address.clone().unwrap().is_err() {
        return null_mut();
    }

    let timeout = if socket_timeout != 0 {
        Some(Duration::new(0, socket_timeout))
    } else {
        None
    };

    let binding = binding_address.map(|binding_address| binding_address.ok().unwrap());
    let connection = connection_address.unwrap();
    let time = Duration::new(0, packet_time);

    let dan = Dan::create(binding, connection, timeout, read_size, write_size, packet_size, time);
    dan.map(|dan| Box::into_raw(Box::new(dan))).unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn dan_destroy(dan: *mut Dan) {
    // Box invokes Drop on Dan when out of scope
    let dan = Box::from_raw(dan);
    Dan::destroy(&*dan.socket);
}

#[no_mangle]
pub unsafe extern "C" fn dan_discover_ip(
    dan: *const Dan,
    packet: *mut c_uchar,
    packet_size: usize,
) -> bool {

    let buffer_packet = &mut Vec::with_capacity(packet_size);
    Dan::discover_ip(&*dan, buffer_packet).map(|_| {
        let packet = from_raw_parts_mut(packet, packet_size);
        packet.clone_from_slice(buffer_packet);
    }).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn dan_read(dan: *const Dan, packet: *mut c_uchar) -> bool {
    let dan = &*dan; // Copies returned Vector into the packet array
    Dan::read(&dan.reader).map(|buffer_packet| {

        let packet = from_raw_parts_mut(packet, dan.socket.packet_size);
        packet.clone_from_slice(&buffer_packet);
    }).is_some()
}

#[no_mangle]
pub unsafe extern "C" fn dan_read_socket(dan: *const Dan) -> bool {
    Dan::read_socket(&(&*dan).read_socket).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn dan_write(dan: *const Dan, packet: *const c_uchar) -> bool {
    let dan = &*dan; // Copies received packet into a new Vector (original untouched)
    let packet = from_raw_parts(packet, dan.socket.packet_size);
    Dan::write(&dan.writer, packet.to_vec())
}

#[no_mangle]
pub unsafe extern "C" fn dan_write_socket(dan: *const Dan) -> bool {
    Dan::write_socket(&(&*dan).write_socket).is_ok()
}
