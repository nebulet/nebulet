#![no_main]
#![feature(
    const_fn,
)]

#[macro_use]
extern crate sip;

mod driver;
mod keyboard;

use std::panic;

#[no_mangle]
pub fn main() {
    panic::set_hook(Box::new(|info| {
        println!("ps2: {}", info);
    }));

    println!("ps2 driver loaded");

    let keyboard = driver::KeyboardDriver::open();

    for key in keyboard.keys() {
        if let keyboard::DecodedKey::Unicode(character) = key {
            print!("{}", character);
        }
    }
}