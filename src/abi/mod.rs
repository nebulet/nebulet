//! The interface between running processes and the kernel
//!

use wasm::runtime::instance::VmCtx;

pub extern fn output_test(arg: usize, vmctx: *const VmCtx) {
    println!("vmctx: {:#x}", vmctx as usize);
    println!("wasm supplied arg = {}", arg);
}
