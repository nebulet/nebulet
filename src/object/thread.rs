use task::{Thread as TaskThread, State};
use object::Process;
use arch::cpu::Local;
use nabi::Result;
use nil::{Ref, HandleRef};
use nil::mem::Bin;
use sync::atomic::Ordering;

/// Represents a thread.
#[derive(HandleRef)]
pub struct Thread {
    thread: TaskThread,
    parent: Ref<Process>,
}

impl Thread {
    pub fn new<F>(parent: Ref<Process>, stack_size: usize, f: F) -> Result<Ref<Thread>>
        where F: FnOnce() + Send + Sync
    {
        let thread = TaskThread::new(stack_size, Bin::new(move || f())?)?;

        Ref::new(Thread {
            thread,
            parent,
        })
    }

    pub fn current() -> Ref<Thread> {
        Local::current_thread()
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

    pub fn parent(&self) -> &Process {
        &self.parent
    }

    pub fn resume(self: &Ref<Self>) {
        assert!({
           let state = self.state();
           state == State::Blocked || state == State::Suspended 
        });
        
        self.set_state(State::Ready);

        Local::schedule_thread(self.clone());
    }

    pub fn exit(self: &Ref<Self>) -> Result<()> {
        
        Ok(())
    }
}
