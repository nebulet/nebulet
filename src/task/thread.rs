use super::stack::Stack;
use arch::context::Context;
use super::ThreadTable;
use super::thread_entry::ThreadEntry;

use nabi::{Result};

/// The current state of a process.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// The thread is currently executing.
    Running,
    /// This thread is not currently running, but it's ready to execute.
    Ready,
    /// The thread has been preempted and cannot be run right now.
    Preempted,
    /// It's dead, Jim.
    Dead,
}

/// A single thread of execution.
#[derive(Debug)]
pub struct Thread {
    state: State,
    ctx: Context,
    stack: Stack,
    entry: extern fn(),
}

impl Thread {
    /// This creates a new thread and adds it to the global thread table.
    pub fn new(stack_size: usize, entry: extern fn()) -> Result<ThreadEntry> {
        let stack = Stack::with_size(stack_size)?;
        let stack_top = stack.top();

        let thread = Thread {
            state: State::Ready,
            ctx: Context::from_rsp(0),
            stack,
            entry,
        };

        let entry = ThreadTable::allocate(thread)?;

        {
            let mut table = ThreadTable::lock();
            table[entry.id()].ctx = Context::init(stack_top, common_thread_entry, entry.id())
        }

        Ok(entry)
    }

    pub unsafe fn switch_to(&mut self, other: &Thread) {
        self.ctx.switch_to(&other.ctx);
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }
}

#[naked]
extern fn common_thread_entry() {
    let thread_entry: ThreadEntry;
    unsafe {
        asm!("pop $0" : "=r"(thread_entry) : : "memory" : "intel", "volatile");
    }

    let func = {
        let thread_table = ThreadTable::lock();
        thread_table[thread_entry.id()].entry
    };

    println!("Starting thread");

    func();

    println!("thread done");
    loop {}
}