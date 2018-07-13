#![no_main]

#[macro_use]
extern crate sip;

mod vga;
use vga::Vga;

#[no_mangle]
pub fn main() {
    println!("vga driver loaded");

    let mut vga = Vga::open();

    vga.clear_screen();

    let chan = sip::Channel::INITIAL;

    let mut buffer = Vec::new();
    buffer.resize(64 * 1024, 0);

    while let (size, Ok(_)) = chan.recv_raw(&mut buffer) {
        vga.write_bytes(&buffer[..size]);
    }
}
