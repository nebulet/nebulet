use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;
use wasm::UserData;

// /// Tests that abi functionality is working.
// pub extern fn output_test(arg: usize, _vmctx: &VmCtx) {
//     println!("wasm supplied arg = {}", arg);
    
//     // println!("calling process name: \"{}\"", vmctx.process.name());
// }

#[nebulet_abi]
pub fn output_test(arg: usize, _: &UserData) -> Result<u32> {
    println!("wasm supplied arg = {}", arg);

    Ok(0)
}

#[nebulet_abi]
pub fn assert_eq(left: u64, right: u64, _: &UserData) -> Result<u32> {
    assert_eq!(left, right);
    Ok(0)
}
