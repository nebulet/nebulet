
pub mod thread;
pub mod process;
pub mod thread_entry;
pub mod scheduler;

use spin::Once;
use arch::lock::{Spinlock, SpinGuard};
use common::table::Table;
pub use self::thread::Thread;
pub use self::thread_entry::ThreadEntry;
pub use self::process::Process;

use self::scheduler::Scheduler;

use nabi::Result;

static THREAD_TABLE: Once<Spinlock<Table<Thread>>> = Once::new();
static SCHEDULER: Once<Scheduler> = Once::new();

extern fn idle_thread_entry(_: usize) {
    loop {
        unsafe { ::arch::interrupt::halt(); }
    }
}

#[inline]
fn scheduler_init() -> Scheduler {
    let idle_thread = Thread::new("idle", 4096, idle_thread_entry, 0)
        .expect("Could not create idle thread");

    let kernel_thread = Thread::new("kernel", 0, idle_thread_entry, 0)
        .unwrap();

    {
        use task::thread::State;
        let mut thread_table = ThreadTable::lock();
        
        thread_table[kernel_thread.id()].set_state(State::Suspended);
    }
    
    Scheduler::new(kernel_thread, idle_thread)
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

    pub fn free(entry: ThreadEntry) -> Result<Thread> {
        ThreadTable::lock()
            .free(entry.id())
    }
}