#![no_main]
#![feature(
    const_fn,
    const_vec_new,
)]

#[macro_use]
extern crate sip;
// use sip::thread;

// mod keyboard;
// mod driver;
// use driver::KeyboardDriver;

use std::panic;

#[no_mangle]
pub fn main() {
    panic::set_hook(Box::new(|info| {
        println!("userspace: {}", info);
    }));

    println!("Hello from wasm!");
}
