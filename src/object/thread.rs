use task::{Thread, State};
use object::ProcessRef;
use arch::cpu::Local;
use nabi::Result;
use nil::{Ref, KernelRef};
use nil::mem::Bin;
use arch::lock::IrqSpinlock;

/// Represents a thread.
#[derive(KernelRef)]
pub struct ThreadRef {
    thread: IrqSpinlock<Thread>,
    parent: Ref<ProcessRef>,
}

impl ThreadRef {
    pub fn new<F>(parent: Ref<ProcessRef>, stack_size: usize, f: F) -> Result<Ref<ThreadRef>>
        where F: FnOnce() + Send + Sync
    {
        let thread = IrqSpinlock::new(Thread::new(stack_size, Bin::new(move || f())?)?);

        Ref::new(ThreadRef {
            thread,
            parent,
        })
    }

    pub fn inner(&self) -> &IrqSpinlock<Thread> {
        &self.thread
    }

    pub fn set_state(&self, state: State) {
        self.thread.lock().state = state;
    }

    pub fn state(&self) -> State {
        self.thread.lock().state
    }

    pub fn parent(&self) -> &ProcessRef {
        &self.parent
    }

    pub fn resume(self: &Ref<Self>) -> Result<()> {
        debug_assert!({
           let state = self.state();
           state == State::Blocked || state == State::Suspended 
        });
        self.set_state(State::Ready);

        Local::current()
            .scheduler
            .push(self.clone());
        
        Ok(())
    }

    pub fn exit(self: &Ref<Self>) -> Result<()> {
        
        Ok(())
    }
}
