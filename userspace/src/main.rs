#![no_main]
#![feature(
    const_fn,
    const_vec_new,
)]

#[macro_use]
extern crate sip;

mod keyboard;
mod vga;

use keyboard::{KeyboardDriver, DecodedKey};

use std::panic;

#[no_mangle]
pub fn main() {
    panic::set_hook(Box::new(|info| {
        println!("userspace: {}", info);
    }));

    println!("in driver");

    let mut vga = vga::Vga::open();
    vga.clear_screen();

    let keyboard = KeyboardDriver::open();

    for key in keyboard.keys() {
        if let DecodedKey::Unicode(character) = key {
            vga.write_bytes(&[character as u8]);
        }
    }
}
