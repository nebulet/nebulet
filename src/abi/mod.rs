//! The interface between running processes and the kernel
//! 

// use context;

use nabi::{Result, Error};

pub extern fn output_test(arg: usize) {
    println!("wasm supplied arg = {}", arg);
}