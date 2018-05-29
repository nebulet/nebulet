use memory::WasmStack;
use arch::context::ThreadContext;
use arch::cpu::Local;
use nabi::{Result, Error};
use nil::mem::Bin;

/// The current state of a process.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// The thread is currently executing.
    Running,
    /// This thread is not currently running, but it's ready to execute.
    /// For example, in the cpu thread queue.
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
    entry: usize,
}

impl Thread {
    pub fn new<F>(stack_size: usize, entry: Bin<F>) -> Result<Thread>
        where F: FnOnce() + Send + Sync
    {
        let stack = WasmStack::allocate(stack_size)
            .ok_or(Error::NO_MEMORY)?;

        let thread = Thread {
            state: State::Suspended,
            ctx: ThreadContext::new(stack.top(), common_thread_entry::<F>),
            stack,
            entry: entry.into_nonnull().as_ptr() as *const () as usize,
        };

        Ok(thread)
    }

    pub unsafe fn swap(&mut self, other: &Thread) {
        self.ctx.swap(&other.ctx);
    }
}

extern fn common_thread_entry<F>()
    where F: FnOnce() + Send + Sync
{
    let thread = unsafe { &mut *Local::current_thread().as_ptr() };

    let f = unsafe { (thread.entry as *const F).read() };
    f();

    thread.state = State::Dead;

    unsafe {
        Local::current()
        .scheduler
        .switch();
    }

    unreachable!();
}
