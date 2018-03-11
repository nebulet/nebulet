
pub mod thread;
pub mod sched;

pub use self::thread::{LockedThread, Thread, ThreadPriority, ThreadFlags, ThreadState};
pub use self::sched::Scheduler;

static mut SCHEDULER: Option<Scheduler> = None;

pub fn init() {
    unsafe {
        SCHEDULER = Some(Scheduler::new());
    }
}

pub fn scheduler() -> &'static Scheduler {
    unsafe {
        if let Some(ref sched) = SCHEDULER {
            sched
        } else {
            panic!("Scheduler not initialized");
        }
    }
}

pub fn resched() {
    scheduler().resched();
}