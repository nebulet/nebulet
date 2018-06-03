// The control flow analysis seems broken when all
// the conditional features are disabled
#![allow(unreachable_code)]

use nabi::{Result, Error};
use core;
use rand_core;
#[cfg(feature="rdseed_entropy")]
use arch::x64::devices::rand::rdseed;
#[cfg(feature="jitter_entropy")]
use arch::x64::devices::rand::jitter;
#[cfg(feature="jitter_entropy")]
use rand::jitter::JitterRng;
#[cfg(feature="rdrand_entropy")]
use arch::x64::devices::rand::rdrand;

#[derive(Debug)]
enum EntropySource {
    #[cfg(feature="rdseed_entropy")]
    RdSeed(rdseed::RdSeed),
    #[cfg(feature="jitter_entropy")]
    Jitter(JitterRng),
    #[cfg(feature="rdrand_entropy")]
    RdRand(rdrand::RdRand),
}


#[derive(Debug)]
pub struct EntropyRng {
    rng: EntropySource,
}

#[cfg(not(any(
            feature="rdseed_entropy",
            feature="jitter_entropy",
            feature="rdrand_entropy")))]
const NOSOURCE_MSG : &'static str
= "No configured source of trusted entropy";

impl EntropyRng {
    pub fn new() -> Result<Self> {
        #[cfg(feature="rdseed_entropy")]
        match rdseed::RdSeed::new() {
            Some(rng) =>
                return Ok(EntropyRng {
                    rng: EntropySource::RdSeed(rng) }),
            None => {},
        };
        #[cfg(feature="jitter_entropy")]
        match jitter::build_jitter() {
            Ok(rng) =>
                return Ok(EntropyRng {
                    rng: EntropySource::Jitter(rng) }),
            Err(_) => {},
        };
        #[cfg(feature="rdrand_entropy")]
        match rdrand::RdRand::new() {
            Some(rng) =>
                return Ok(EntropyRng {
                    rng: EntropySource::RdRand(rng) }),
            None => {},
        };
        return Err(Error::UNAVAILABLE);
    }
}

impl rand_core::RngCore for EntropyRng {
        fn next_u32(&mut self) -> u32 {
            match self.rng {
                #[cfg(feature="rdseed_entropy")]
                EntropySource::RdSeed(ref mut rng) =>
                    return rng.next_u32(),
                #[cfg(feature="jitter_entropy")]
                EntropySource::Jitter(ref mut rng) =>
                    return rng.next_u32(),
                #[cfg(feature="rdrand_entropy")]
                EntropySource::RdRand(ref mut rng) =>
                    return rng.next_u32(),
            }
            #[cfg(not(any(
                        feature="rdseed_entropy",
                        feature="jitter_entropy",
                        feature="rdrand_entropy")))]
            panic!(NOSOURCE_MSG);
        }

        fn next_u64(&mut self) -> u64 {
            match self.rng {
                #[cfg(feature="rdseed_entropy")]
                EntropySource::RdSeed(ref mut rng) =>
                    return rng.next_u64(),
                #[cfg(feature="jitter_entropy")]
                EntropySource::Jitter(ref mut rng) =>
                    return rng.next_u64(),
                #[cfg(feature="rdrand_entropy")]
                EntropySource::RdRand(ref mut rng) =>
                    return rng.next_u64(),
            }
            #[cfg(not(any(
                        feature="rdseed_entropy",
                        feature="jitter_entropy",
                        feature="rdrand_entropy")))]
            panic!(NOSOURCE_MSG);
        }

        fn fill_bytes(&mut self, _dest: &mut [u8]) {
            match self.rng {
                #[cfg(feature="rdseed_entropy")]
                EntropySource::RdSeed(ref mut rng) =>
                    return rng.fill_bytes(_dest),
                #[cfg(feature="jitter_entropy")]
                EntropySource::Jitter(ref mut rng) =>
                    return rng.fill_bytes(_dest),
                #[cfg(feature="rdrand_entropy")]
                EntropySource::RdRand(ref mut rng) =>
                    return rng.fill_bytes(_dest),
            }
            #[cfg(not(any(
                        feature="rdseed_entropy",
                        feature="jitter_entropy",
                        feature="rdrand_entropy")))]
            panic!(NOSOURCE_MSG);
        }

        fn try_fill_bytes(&mut self, _dest: &mut [u8])
            -> core::result::Result<(), rand_core::Error> {
            match self.rng {
                #[cfg(feature="rdseed_entropy")]
                EntropySource::RdSeed(ref mut rng) =>
                    return rng.try_fill_bytes(_dest),
                #[cfg(feature="jitter_entropy")]
                EntropySource::Jitter(ref mut rng) =>
                    return rng.try_fill_bytes(_dest),
                #[cfg(feature="rdrand_entropy")]
                EntropySource::RdRand(ref mut rng) =>
                    return rng.try_fill_bytes(_dest),
            }
            #[cfg(not(any(
                        feature="rdseed_entropy",
                        feature="jitter_entropy",
                        feature="rdrand_entropy")))]
            return Err(rand_core::Error::new(
                    rand_core::ErrorKind::Unavailable,
                    NOSOURCE_MSG));
        }
}

