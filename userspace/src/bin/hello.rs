#![no_main]

#[macro_use]
extern crate userspace;

#[no_mangle]
pub fn main() {
    println!("Checking in from `hello.wasm`");
}
