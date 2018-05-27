use spin::RwLock;
pub use core::time::Duration;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// Kernel start time, measured in (seconds, nanoseconds) since Unix epoch
pub static START: RwLock<(u64, u32)> = RwLock::new((0, 0));

/// Return the start time of the kernel
pub fn start() -> SystemTime {
    let (secs, nanos) = *START.read();
    SystemTime(Duration::new(secs, nanos))
}

/// Return the up time of the kernel in nanoseconds
#[inline]
fn monotonic() -> u64 {
    ::arch::x64::devices::high_precision_timer::now()
}

/// Returns the realtime of the kernel
#[inline]
fn realtime() -> (u64, u32) {
    let offset = monotonic();
    let start = *START.read();
    let sum = start.1 as u64 + offset;
    (start.0 + sum / 1_000_000_000, (sum % 1_000_000_000) as u32)
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Instant(Duration);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct SystemTime(Duration);

pub const UNIX_EPOCH: SystemTime = SystemTime(Duration::from_secs(0));
pub const INSTANT_INIT: Instant = Instant(Duration::from_secs(0));

impl Instant {
    pub fn now() -> Instant {
        let nanos = monotonic();
        Instant(Duration::new(nanos / 1_000_000_000, (nanos % 1_000_000_000) as u32))
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0 - earlier.0
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, other: Duration) -> Instant {
        Instant(self.0 + other)
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: Duration) -> Instant {
        Instant(self.0 - other)
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

impl SystemTime {
    pub fn new() -> SystemTime {
        let (secs, nanos) = realtime();
        SystemTime(Duration::new(secs, nanos))
    }

    pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, Duration> {
        self.0.checked_sub(other.0).ok_or_else(|| other.0 - self.0)
    }

    pub fn add_duration(&self, other: &Duration) -> SystemTime {
        SystemTime(self.0 + *other)
    }

    pub fn sub_duration(&self, other: &Duration) -> SystemTime {
        SystemTime(self.0 - *other)
    }
}