//! Expose randomness based on CPU jitter

use arch::devices::high_precision_timer;
use nabi::{Error, Result};
use rand::jitter::JitterRng;
use rand_core::RngCore;

pub fn build_jitter() -> Result<JitterRng> {
    let mut rng = JitterRng::new_with_timer(high_precision_timer::now);
    let rounds = rng.test_timer().map_err(|_| Error::UNAVAILABLE)?;
    rng.set_rounds(rounds);
    let _ = rng.next_u64();
    return Ok(rng);
}
