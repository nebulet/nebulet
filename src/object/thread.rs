use task::{Thread as TaskThread, State};
use object::Process;
use arch::cpu::Local;
use nabi::Result;
use nil::{Ref, HandleRef};
use nil::mem::Bin;
use sync::atomic::Ordering;
use dpc;

/// Represents a thread.
#[derive(HandleRef)]
pub struct Thread {
    thread: TaskThread,
    parent: Ref<Process>,
}

impl Thread {
    pub fn new<F>(stack_size: usize, f: F) -> Result<Ref<Thread>>
        where F: FnOnce() + Send + Sync
    {
        Self::new_with_parent(unsafe { Ref::dangling() }, stack_size, f)
    }

    pub fn new_with_parent<F>(parent: Ref<Process>, stack_size: usize, f: F) -> Result<Ref<Thread>>
        where F: FnOnce() + Send + Sync
    {
        let thread = TaskThread::new(stack_size, Bin::new(move || f())?)?;

        let t = Ref::new(Thread {
            thread,
            parent,
        })?;

        t.inc_ref();

        Ok(t)
    }
    
    pub fn current() -> &'static Thread {
        unsafe { &*Local::current_thread() }
    }

    /// Yield the current thread.
    pub fn yield_now() {
        unsafe {
            Local::context_switch();
        }
    }

    pub fn inner(&self) -> &TaskThread {
        &self.thread
    }

    pub fn set_state(&self, state: State) {
        self.thread.state.store(state, Ordering::Release)
    }

    pub fn state(&self) -> State {
        self.thread.state.load(Ordering::Acquire)
    }

    pub fn parent(&self) -> &Ref<Process> {
        &self.parent
    }

    pub fn resume(&self) {
        assert!({
           let state = self.state();
           state == State::Blocked || state == State::Suspended 
        });
        
        self.set_state(State::Running);

        Local::schedule_thread(self);
    }

    pub fn exit(self: Ref<Self>) -> Result<()> {
        /// Killing a thread has to use
        /// a deferred procedure call
        /// because if the exit method decremented
        /// the ref itself, the thread (including its stack)
        /// would get deallocated while it was running, and
        /// this would instantly crash.
        fn kill_thread(arg: usize) {
            let thread = unsafe { Ref::from_raw(arg as *const Thread) };
            thread.dec_ref();
        }

        self.set_state(State::Dead);

        dpc::queue(Ref::into_raw(self) as _, kill_thread);

        Ok(())
    }
}
