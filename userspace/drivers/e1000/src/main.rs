#![no_main]

#[macro_use]
extern crate sip;

mod device;

use self::device::Intel8254x;
use std::panic;

#[no_mangle]
pub fn main() {
    panic::set_hook(Box::new(|info| {
        println!("e1000: {}", info);
    }));

    let _device = unsafe { Intel8254x::new(0xfebc0000).unwrap() };
}