use object::ProcessRef;
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

// /// Tests that abi functionality is working.
// pub extern fn output_test(arg: usize, _vmctx: &VmCtx) {
//     println!("wasm supplied arg = {}", arg);
    
//     // println!("calling process name: \"{}\"", vmctx.process.name());
// }

#[nebulet_abi]
pub fn output_test(arg: usize, _process: &ProcessRef) -> Result<u32> {
    println!("wasm supplied arg = {}", arg);

    Ok(0)
}

// #[nebulet_abi]
pub fn swap_stack(_: &ProcessRef) -> Result<u32> {
    // use memory::WasmStack;

    // let mut stack = WasmStack::allocate(1024 * 1024)?;


    Ok(0)
}
