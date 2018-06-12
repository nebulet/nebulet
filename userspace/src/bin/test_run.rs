#![no_main]

#[macro_use]
extern crate sip;

// use sip::Channel;
// use std::str;

static HELLO: &[u8] = b"Hello from wasm!";

fn clear_screen(buffer: &mut [u16]) {
    for byte in buffer {
        *byte = 0;
    }
}

#[no_mangle]
pub fn main() {
    println!("Mapping vga buffer.");
    let vga_buffer = sip::physical_map::<[u16; 80 * 25]>(0xb8000).unwrap();

    clear_screen(vga_buffer);

    for (i, &byte) in HELLO.iter().enumerate() {
        vga_buffer[i] = 0xe << 8 | byte as u16;
    }

    loop {}
}
