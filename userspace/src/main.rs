#![no_main]
#![feature(const_fn)]

#[macro_use]
extern crate sip;

mod keyboard;
mod driver;

use driver::KeyboardDriver;

#[no_mangle]
pub fn main() {
    // let _driver = KeyboardDriver::new();

    let event = match sip::Event::create() {
        Ok(event) => event,
        Err(err) => {
            println!("`Event::create` error: {:?}", err);
            unimplemented!()
        },
    };

    println!("trigger: {:?}", event.trigger());

    println!("Going to wait.");
    event.wait();
    println!("after wait");

    loop {}
}
