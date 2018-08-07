use wasm::UserData;
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

#[nebulet_abi]
pub fn physical_map(phys_address: u64, count: u32, data: &UserData) -> Result<u32> {
    let memory = &data.instance.memories[0];

    memory.physical_map(phys_address, count as usize)
        .map(|addr| addr as u32)
}
