#![no_main]
#![feature(const_fn)]

#[macro_use]
extern crate sip;

mod keyboard;
mod driver;

use driver::KeyboardDriver;

#[no_mangle]
pub fn main() {
    let _driver = KeyboardDriver::new();

    loop {}
}
