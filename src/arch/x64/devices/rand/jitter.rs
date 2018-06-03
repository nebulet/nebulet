//! Expose randomness based on CPU jitter

use rand_core::RngCore;
use rand::jitter::JitterRng;
use arch::devices::high_precision_timer;
use nabi::{Result, Error};

pub fn build_jitter() -> Result<JitterRng> {
    let mut rng = JitterRng::new_with_timer(high_precision_timer::now);
    let rounds = rng.test_timer().map_err(|_| Error::UNAVAILABLE)?;
    rng.set_rounds(rounds);
    let _ = rng.next_u64();
    return Ok(rng);
}
