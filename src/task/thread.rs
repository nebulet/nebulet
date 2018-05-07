use memory::WasmStack;
use arch::context::ThreadContext;
use super::ThreadTable;
use super::thread_entry::ThreadEntry;
use super::GlobalScheduler;

use nabi::{Result, Error};

/// The current state of a process.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// The thread is currently executing.
    Running,
    /// This thread is not currently running, but it's ready to execute.
    Ready,
    /// The thread has been suspended and cannot be run right now.
    Suspended,
    /// It's dead, Jim.
    Dead,
}

/// A single thread of execution.
#[derive(Debug)]
pub struct Thread {
    state: State,
    ctx: ThreadContext,
    stack: WasmStack,
    entry: extern fn(usize),
    arg: usize,
}

impl Thread {
    /// This creates a new thread and adds it to the global thread table.
    pub fn new(stack_size: usize, entry: extern fn(usize), arg: usize) -> Result<ThreadEntry> {
        let stack = WasmStack::allocate(stack_size)
            .ok_or(Error::NO_MEMORY)?;

        let thread = Thread {
            state: State::Ready,
            ctx: ThreadContext::new(stack.top(), common_thread_entry),
            stack,
            entry,
            arg,
        };

        let entry = ThreadTable::allocate(thread)?;

        {
            let mut table = ThreadTable::lock();
            table[entry.id()].ctx.rbx = entry.id();
        }

        Ok(entry)
    }

    pub unsafe fn swap(&mut self, other: &Thread) {
        self.ctx.swap(&other.ctx);
    }
    
    pub fn state(&self) -> State {
        self.state
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }
}

extern fn common_thread_entry() {
    let thread_entry: ThreadEntry;
    unsafe {
        asm!("" : "={rbx}"(thread_entry) : : "memory" : "intel", "volatile");
    }

    let (func, arg) = {
        let thread_table = ThreadTable::lock();
        let thread = &thread_table[thread_entry.id()];
        (thread.entry, thread.arg)
    };

    func(arg);

    {
        let mut thread_table = ThreadTable::lock();
        let thread = &mut thread_table[thread_entry.id()];

        thread.set_state(State::Dead);
    }

    GlobalScheduler::switch();

    unreachable!();
}