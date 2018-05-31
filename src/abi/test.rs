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

#[nebulet_abi]
pub fn write_i64(offset: u32, value: u64, process: &ProcessRef) -> Result<u32> {
    let mut instance = process.instance().write();
    let wasm_memory = &mut instance.memories[0];
    let value_ref = wasm_memory.carve_mut(offset)?;

    *value_ref = value;

    Ok(0)
}

#[nebulet_abi]
pub fn assert_eq(left: u64, right: u64, _: &ProcessRef) -> Result<u32> {
    assert_eq!(left, right);
    Ok(0)
}
