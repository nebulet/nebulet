//! The interface between running processes and the kernel
//!

pub extern fn output_test(arg: usize, vmctx: usize) {
    println!("wasm supplied arg = {}", arg);
    println!("vmctx = {:#x}", vmctx);
}
