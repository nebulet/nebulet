use arch::lock::{IrqGuard, IrqLock};
use arch::x64::devices::rand::entropy::EntropyRng;
use nabi::Result;
use rand::prng::hc128::Hc128Core;
use rand::rngs::adapter::ReseedingRng;
use rand_core::SeedableRng;

const GLOBAL_RNG_RESEED_THRESHOLD: u64 = 32 * 1024 * 1024; // 32 MiB

struct GlobalRng {
    rng: IrqLock<Option<Result<ReseedingRng<Hc128Core, EntropyRng>>>>,
}

static GLOBAL_RNG: GlobalRng = GlobalRng {
    rng: IrqLock::new(None),
};

fn new_global_rng() -> Result<ReseedingRng<Hc128Core, EntropyRng>> {
    let mut entropy = EntropyRng::new()?;
    let lowrng = Hc128Core::from_rng(&mut entropy).expect("Failed to initialize Hc128Core");
    return Ok(ReseedingRng::new(
        lowrng,
        GLOBAL_RNG_RESEED_THRESHOLD,
        entropy,
    ));
}

pub fn with_global_rng<T, F>(f: F) -> Result<T>
where
    F: FnOnce(&mut ReseedingRng<Hc128Core, EntropyRng>) -> T,
{
    let mut guard: IrqGuard<_> = GLOBAL_RNG.rng.lock();
    let rng_res = guard.get_or_insert_with(new_global_rng);
    match rng_res {
        &mut Err(ref e) => Err(e.clone()),
        &mut Ok(ref mut v) => Ok(f(v)),
    }
}
