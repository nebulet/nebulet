use memory::WasmStack;
use arch::context::ThreadContext;
use arch::cpu::Local;

use core::ops::{Deref, DerefMut};
use alloc::boxed::Box;
// use super::GlobalScheduler;

use nabi::{Result, Error};

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ThreadRef {
    ptr: *mut Thread,
}

unsafe impl Send for ThreadRef {}
unsafe impl Sync for ThreadRef {}

impl ThreadRef {
    fn from_box(b: Box<Thread>) -> ThreadRef {
        ThreadRef {
            ptr: Box::into_raw(b),
        }
    }

    fn from_thread(thread: Thread) -> ThreadRef {
        Self::from_box(Box::new(thread))
    }

    /// This will destroy the thread and
    /// return true if the thread is `Dead`.
    /// Else, will return false.
    pub unsafe fn destroy(self) -> bool {
        let thread: &Thread = &*self;

        if thread.state == State::Dead {
            let _ = Box::from_raw(self.ptr);
            true
        } else {
            false
        }
    }

    /// This adds this thread to the run queue.
    pub fn resume(self) -> Result<()> {
        let local = Local::current();
        local.scheduler.push(self);
        Ok(())
    }
}

impl Deref for ThreadRef {
    type Target = Thread;
    fn deref(&self) -> &Thread {
        unsafe {
            &*self.ptr
        }
    }
}

impl DerefMut for ThreadRef {
    fn deref_mut(&mut self) -> &mut Thread {
        unsafe {
            &mut *self.ptr
        }
    }
}

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
#[derive(Debug)]
pub struct Thread {
    pub state: State,
    ctx: ThreadContext,
    stack: WasmStack,
    entry: extern fn(usize),
    arg: usize,
}

impl Thread {
    /// This creates a new thread and adds it to the global thread table.
    pub fn new(stack_size: usize, entry: extern fn(usize), arg: usize) -> Result<ThreadRef> {
        let stack = WasmStack::allocate(stack_size)
            .ok_or(Error::NO_MEMORY)?;

        let thread = Thread {
            state: State::Ready,
            ctx: ThreadContext::new(stack.top(), common_thread_entry),
            stack,
            entry,
            arg,
        };

        // TODO: Find a more platform independent way
        // of doing this.

        let mut thread_ref = ThreadRef::from_thread(thread);

        thread_ref.ctx.rbx = &mut *thread_ref as *mut Thread as usize;

        Ok(thread_ref)
    }

    pub unsafe fn swap(&mut self, other: &Thread) {
        self.ctx.swap(&other.ctx);
    }
}

extern fn common_thread_entry() {
    let thread: &mut Thread;
    unsafe {
        asm!("" : "={rbx}"(thread) : : "memory" : "intel", "volatile");
    }

    (thread.entry)(thread.arg);

    thread.state = State::Dead;

    Local::current()
        .scheduler
        .switch();

    unreachable!();
}