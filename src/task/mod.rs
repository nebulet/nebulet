
pub mod thread;
pub mod sched;

pub use self::thread::{LockedThread, Thread, ThreadPriority, ThreadFlags, ThreadState};
pub use self::sched::Scheduler;

use spin::{Once, RwLock, RwLockWriteGuard, RwLockReadGuard};

static SCHEDULER: Once<Scheduler> = Once::new();

fn init_scheduler() -> Scheduler {
    Scheduler::new()
}

pub fn scheduler() -> &'static Scheduler {
    SCHEDULER.call_once(init_scheduler)
}

pub fn resched() {
    scheduler().resched();
}