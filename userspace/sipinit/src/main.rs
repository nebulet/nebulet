#![no_main]

#[macro_use]
extern crate sip;
mod tar;

use tar::Tar;

use std::panic;

#[no_mangle]
pub fn main() {
    panic::set_hook(Box::new(|info| {
        println!("sipinit: {}", info);
    }));

    println!("in sipinit");

    let init_chan = sip::Channel::INITIAL;

    let init_fs: Vec<u8> = init_chan.flat_map(|v| v).collect();

    let tar = Tar::load(&init_fs);

    for file in tar.iter() {
        println!("path: {:?}", file.path);
    }

    // let keyboard_driver = tar.iter().find(|file| file.path == "e1000.wasm").unwrap();

    // let (_tx, rx) = sip::Channel::create().unwrap();

    // // tx.send(b"hello, world").unwrap();

    // launch(keyboard_driver.data, rx);
}

fn launch(wasm_data: &[u8], chan: sip::ReadChannel) {
    println!("compiling");
    let wasm = sip::Wasm::compile(wasm_data).unwrap();
    println!("launching process");
    let process = sip::Process::create(wasm, chan).unwrap();
    process.start().unwrap();
}