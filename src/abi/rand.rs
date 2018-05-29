use object::ProcessRef;
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;
use arch::x64::devices::rand::RdRand;
use rand_core::RngCore;

fn get_rdrand() -> Result<RdRand> {
    RdRand::new().ok_or(Error::UNAVAILABLE)
}

static mut RDRAND : Option<Result<RdRand>> = None;

#[nebulet_abi]
pub fn random_fill(buffer_offset: u32, buffer_size: u32, process: &ProcessRef) -> Result<u32> {
    let rdrand;
    unsafe {
        rdrand = RDRAND.get_or_insert_with(|| get_rdrand());
    }
    match rdrand {
        Ok(ref mut v) => {
            let mut instance = process.instance().write();
            let memory = &mut instance.memories[0];

            let buffer = memory.carve_slice_mut(buffer_offset, buffer_size)
                .ok_or(Error::INVALID_ARG)?;
            v.fill_bytes(buffer);
            Ok(0)
        },
        // XXX Find a way to return the existing error
        _ => Err(Error::UNAVAILABLE),
    }
}
