use task::{Thread, State};
use arch::cpu::Local;
use nabi::Result;
use nil::{Ref, KernelRef};
use alloc::boxed::Box;
use spin::RwLock;

/// Represents a thread.
#[derive(KernelRef)]
pub struct ThreadRef {
    thread: RwLock<Thread>,
}

impl ThreadRef {
    pub fn new<F>(stack_size: usize, f: F) -> Result<Ref<ThreadRef>>
        where F: FnOnce() + Send + Sync + 'static
    {
        let thread = RwLock::new(Thread::new(stack_size, Box::new(move || f()))?);

        Ok(ThreadRef {
            thread,
        }.into())
    }

    pub fn set_state(&self, state: State) {
        self.thread.write().state = state;
    }

    pub fn state(&self) -> State {
        self.thread.read().state
    }

    pub fn resume(self: &Ref<Self>) -> Result<()> {
        // increase ref count so that the 
        // thread pointer doesn't get deallocated
        // while being used in the scheduler.
        self.inc_ref();

        let thread_ptr = (&mut *self.thread.write()) as *mut Thread;

        Local::current()
            .scheduler
            .push(thread_ptr);
        
        Ok(())
    }

    pub fn exit(self: Ref<Self>) -> Result<()> {
        // Now that the thread pointer
        // is no longer being used,
        // we can decrement the reference
        // count.
        self.dec_ref();

        Ok(())
    }
}
