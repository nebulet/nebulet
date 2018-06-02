#![no_std]
#![no_main]

extern crate userspace;

#[no_mangle]
pub fn main() {
    userspace::print("Hello world!");
}