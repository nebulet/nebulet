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
pub fn random_u32(_process: &ProcessRef) -> Result<u32> {
    let rdrand;
    unsafe {
        rdrand = RDRAND.get_or_insert_with(|| get_rdrand());
    }
    match rdrand {
        Ok(ref mut v) => Ok(v.next_u32()),
        // XXX Find a way to return the existing error
        _ => Err(Error::UNAVAILABLE),
    }
}
