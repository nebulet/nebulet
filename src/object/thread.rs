use task::{Thread as TaskThread, State};
use object::Process;
use arch::cpu::Local;
use nabi::Result;
use nil::{Ref, KernelRef};
use nil::mem::Bin;
use arch::lock::IrqSpinlock;

/// Represents a thread.
#[derive(KernelRef)]
pub struct Thread {
    thread: IrqSpinlock<TaskThread>,
    parent: Ref<Process>,
}

impl Thread {
    pub fn new<F>(parent: Ref<Process>, stack_size: usize, f: F) -> Result<Ref<Thread>>
        where F: FnOnce() + Send + Sync
    {
        let thread = IrqSpinlock::new(TaskThread::new(stack_size, Bin::new(move || f())?)?);

        Ref::new(Thread {
            thread,
            parent,
        })
    }

    pub fn inner(&self) -> &IrqSpinlock<TaskThread> {
        &self.thread
    }

    pub fn set_state(&self, state: State) {
        self.thread.lock().state = state;
    }

    pub fn state(&self) -> State {
        self.thread.lock().state
    }

    pub fn parent(&self) -> &Process {
        &self.parent
    }

    pub fn resume(self: &Ref<Self>) -> Result<()> {
        debug_assert!({
           let state = self.state();
           state == State::Blocked || state == State::Suspended 
        });
        self.set_state(State::Ready);

        Local::schedule_thread(self.clone());
        
        Ok(())
    }

    pub fn exit(self: &Ref<Self>) -> Result<()> {
        
        Ok(())
    }
}
