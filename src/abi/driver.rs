use object::{ProcessRef};
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

#[nebulet_abi]
pub fn physical_map(phys_address: u64, count: u32, process: &ProcessRef) -> Result<u32> {
    let mut instance = process.instance().write();
    let memory = &mut instance.memories[0];

    memory.physical_map(phys_address, count as usize)
        .map(|addr| addr as u32)
}
