use object::ProcessRef;
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;
use arch::x64::devices::rand::rdrand::RdRand;
use arch::x64::devices::rand::seeded;
use rand_core::RngCore;

fn get_rdrand() -> Result<RdRand> {
    RdRand::new().ok_or(Error::UNAVAILABLE)
}

static mut RDRAND : Option<Result<RdRand>> = None;

/// Provides random bytes
/// No guarantee is made that the random bytes are of cryptographic
/// quality, or that they were seeded from a good entropy pool.
/// This currently requires the rdrand instruction, which is fast
/// but not supported everywhere.
#[nebulet_abi]
pub fn random_fill(buffer_offset: u32, buffer_size: u32, process: &ProcessRef) -> Result<u32> {
    let rdrand;
    unsafe {
        rdrand = RDRAND.get_or_insert_with(get_rdrand);
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
        Err(ref e) => Err(e.clone()),
    }
}

/// Provides random bytes
/// Assuming the entropy source configured using *_entropy Cargo
/// features is trusted, this provides bytes of cryptographic
/// quality.
/// To provide good performance, this should be used to seed a prng
/// local to the WASM process.
#[nebulet_abi]
pub fn cprng_fill(
    buffer_offset: u32, buffer_size: u32, process: &ProcessRef)
    -> Result<u32>
{
    let mut instance = process.instance().write();
    let memory = &mut instance.memories[0];
    let buffer = memory.carve_slice_mut(buffer_offset, buffer_size)
        .ok_or(Error::INVALID_ARG)?;
    seeded::with_global_rng(|rng| rng.fill_bytes(buffer))?;
    Ok(0)
}

