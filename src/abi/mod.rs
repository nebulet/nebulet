//! The interface between running processes and the kernel
//!

pub extern fn output_test(arg: usize) {
    println!("wasm supplied arg = {}", arg);
}
