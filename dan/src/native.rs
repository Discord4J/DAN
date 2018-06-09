use socket::DanSocket;
use std::ffi::CStr;
use std::io::Write;
use std::ptr::null_mut;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::time::Duration;

#[no_mangle]
pub extern "C" fn dan_create(
    binding_address: *const i8,
    connection_address: *const i8,
    read_size: usize,
    write_size: usize,
    packet_size: usize,
    packet_time: u32,
) -> *mut DanSocket {

    let binding_address = match binding_address.is_null() {
        true => None,
        _ => Some(unsafe { CStr::from_ptr(binding_address) })
    };

    let connection_address = unsafe { CStr::from_ptr(connection_address) };
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

    let binding = binding_address.map(|address| address.ok().unwrap());
    let connection = connection_address.unwrap();
    let time = Duration::new(0, packet_time);

    let dan = DanSocket::create(binding, connection, read_size, write_size, packet_size, time);
    dan.map(|dan| Box::into_raw(Box::new(dan))).unwrap_or(null_mut())
}

#[no_mangle]
pub extern "C" fn dan_destroy(dan: *mut DanSocket) {
    // Box invokes Drop for DanSocket when dropped
    unsafe { Box::from_raw(dan) };
}

#[no_mangle]
pub extern "C" fn dan_discover_ip(
    dan: *const DanSocket,
    timeout: u32,
    packet: *mut u8,
    packet_size: usize,
) -> bool {

    let dan = unsafe { &*dan };
    let packet = unsafe { from_raw_parts_mut(packet, packet_size) };
    dan.discover_ip(Duration::new(0, timeout), packet).is_ok()
}

#[no_mangle]
pub extern "C" fn dan_read(dan: *const DanSocket, packet: *mut u8) -> bool {
    let dan = unsafe { &*dan }; // Copies returned Vector into packet array
    dan.read().and_then(|buffer_packet| {

        let mut packet = unsafe { from_raw_parts_mut(packet, dan.packet_size) };
        packet.write_all(buffer_packet.as_slice()).ok()
    }).is_some()
}

#[no_mangle]
pub extern "C" fn dan_write(dan: *const DanSocket, packet: *const u8) -> bool {
    let dan = unsafe { &*dan }; // Copies received packet into a new Vector
    let packet = unsafe { from_raw_parts(packet, dan.packet_size) };
    let buffer_packet = &mut Vec::with_capacity(dan.packet_size);
    buffer_packet.clone_from_slice(packet);
    dan.write(buffer_packet.clone())
}

#[no_mangle]
pub extern "C" fn dan_read_all(dan: *const DanSocket) -> bool {
    // If this returns successfully, dan is unsafe to use after
    unsafe { &*dan }.read_all().is_ok()
}

#[no_mangle]
pub extern "C" fn dan_write_all(dan: *const DanSocket) -> bool {
    // If this returns successfully, dan is unsafe to use after
    unsafe { &*dan }.write_all().is_ok()
}
