//! The interface between running processes and the kernel
//!

use wasm::runtime::instance::VmCtx;

pub extern fn output_test(arg: usize, vmctx: &VmCtx) {
    println!("wasm supplied arg = {}", arg);
    
    println!("calling process name: \"{}\"", vmctx.process.name());
}
