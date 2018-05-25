use memory::WasmStack;
use arch::context::ThreadContext;
use arch::cpu::Local;
use alloc::boxed::{Box, FnBox};
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
    /// The thread is blocked.
    Blocked,
    /// It's dead, Jim.
    Dead,
}

/// A single thread of execution.
#[allow(dead_code)]
pub struct Thread {
    pub state: State,
    ctx: ThreadContext,
    stack: WasmStack,
    entry: Option<Box<FnBox() + Send + Sync + 'static>>,
}

impl Thread {
    pub fn new(stack_size: usize, entry: Box<FnBox() + Send + Sync + 'static>) -> Result<Thread> {
        let stack = WasmStack::allocate(stack_size)
            .ok_or(Error::NO_MEMORY)?;

        let thread = Thread {
            state: State::Ready,
            ctx: ThreadContext::new(stack.top(), common_thread_entry),
            stack,
            entry: Some(entry),
        };

        Ok(thread)
    }

    pub unsafe fn swap(&mut self, other: &Thread) {
        self.ctx.swap(&other.ctx);
    }
}

extern fn common_thread_entry() {
    let thread = unsafe { &mut *Local::current_thread().as_ptr() };

    let f: Box<FnBox()> = thread.entry.take().unwrap();
    f();

    thread.state = State::Dead;

    unsafe {
        Local::current()
        .scheduler
        .switch();
    }

    unreachable!();
}
