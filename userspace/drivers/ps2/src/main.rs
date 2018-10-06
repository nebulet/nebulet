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

    let keyboard = driver::KeyboardDriver::open();

    println!("ps2 driver loaded");

    for key in keyboard.keys() {
        println!("{:?}", key);
        if let keyboard::DecodedKey::Unicode(character) = key {
            print!("{}", character);
        }
    }
}