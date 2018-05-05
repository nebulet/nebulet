
pub mod thread;
pub mod process;
pub mod thread_entry;
pub mod scheduler;
mod stack;

use spin::Once;
use arch::lock::{Spinlock, SpinGuard};
use common::table::Table;
use self::thread::Thread;
use self::thread_entry::ThreadEntry;
use self::scheduler::Scheduler;

use nabi::Result;

static THREAD_TABLE: Once<Spinlock<Table<Thread>>> = Once::new();
static SCHEDULER: Once<Scheduler> = Once::new();

extern fn idle_thread_entry() {
    loop {
        unsafe { ::arch::interrupt::halt(); }
    }
}

#[inline]
fn scheduler_init() -> Scheduler {
    let idle_thread = Thread::new(512, idle_thread_entry)
        .expect("Could not create idle thread");
    
    Scheduler::new(idle_thread)
}

#[inline]
fn thread_table_init() -> Spinlock<Table<Thread>> {
    Spinlock::new(Table::new())
}

pub struct GlobalScheduler;

impl GlobalScheduler {
    fn get() -> &'static Scheduler {
        SCHEDULER
            .call_once(scheduler_init)
    }

    pub fn push(entry: ThreadEntry) {
        GlobalScheduler::get()
            .push(entry);
    }

    pub fn switch() {
        GlobalScheduler::get()
            .switch();
    }
}

pub struct ThreadTable;

impl ThreadTable {
    pub fn lock() -> SpinGuard<'static, Table<Thread>> {
        THREAD_TABLE
            .call_once(thread_table_init)
            .lock()
    }

    pub fn allocate(thread: Thread) -> Result<ThreadEntry> {
        let index = ThreadTable::lock()
            .allocate(thread)?;

        Ok(ThreadEntry(index))
    }

    pub fn free(entry: ThreadEntry) -> Result<()> {
        ThreadTable::lock()
            .free(entry.id())
    }
}