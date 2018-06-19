use memory::WasmStack;
use arch::context::ThreadContext;
use arch::cpu::Local;
use nabi::{Result, Error};
use nil::mem::Bin;
use sync::atomic::{Atomic, Ordering};
use core::cell::UnsafeCell;

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

unsafe impl Sync for Thread {}

/// A single thread of execution.
#[allow(dead_code)]
pub struct Thread {
    pub state: Atomic<State>,
    ctx: UnsafeCell<ThreadContext>,
    pub stack: UnsafeCell<WasmStack>,
    entry: usize,
}

impl Thread {
    pub fn new<F>(stack_size: usize, entry: Bin<F>) -> Result<Thread>
        where F: FnOnce() + Send + Sync
    {
        let stack = WasmStack::allocate(stack_size)
            .ok_or(Error::NO_MEMORY)?;

        let thread = Thread {
            state: Atomic::new(State::Suspended),
            ctx: UnsafeCell::new(ThreadContext::new(stack.top(), common_thread_entry::<F>)),
            stack: UnsafeCell::new(stack),
            entry: entry.into_nonnull().as_ptr() as *const () as usize,
        };

        Ok(thread)
    }

    pub unsafe fn swap(&self, other: &Thread) {
        let ctx = &mut*self.ctx.get();
        let other = &*other.ctx.get();
        ctx.swap(other);
    }
}

extern fn common_thread_entry<F>()
    where F: FnOnce() + Send + Sync
{
    let current_thread_ref = Local::current_thread();

    let f = {
        let thread = current_thread_ref.inner();

        unsafe { (thread.entry as *const F).read() }  
    };

    f();

    {
        let thread = current_thread_ref.inner();

        thread.state.store(State::Dead, Ordering::SeqCst);
    }

    unsafe {
        Local::context_switch();
    }

    unreachable!();
}
