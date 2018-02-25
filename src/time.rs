use spin::Mutex;

/// Kernel start time, measured in (seconds, nanoseconds) since Unix epoch
pub static START: Mutex<(u64, u64)> = Mutex::new((0, 0));
/// Kernel up time, measured in (seconds, nanoseconds) since `time::START`
pub static OFFSET: Mutex<(u64, u64)> = Mutex::new((0, 0));

pub fn monotonic() -> (u64, u64) {
    *OFFSET.lock()
}

pub fn realtime() -> (u64, u64) {
    let offset = monotonic();
    let start = *START.lock();
    let sum = start.1 + offset.1;
    (start.0 + offset.0 + sum / 1_000_000_000, sum % 1_000_000_000)
}

pub fn start() -> (u64, u64) {
    *START.lock()
}