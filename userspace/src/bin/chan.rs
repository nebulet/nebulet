#![no_main]

#[macro_use]
extern crate userspace;

#[no_mangle]
pub fn main() {
    println!("Creating channel.");
    let (handle0, handle1) = userspace::channel_create().unwrap();

    println!("handles: ({}, {})", handle0, handle1);
}
