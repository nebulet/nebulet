use nil::{Ref, KernelRef};
use object::Thread;
use task::State;
use nabi::{Result};

#[derive(KernelRef)]
pub struct EventRef {
    thread: Ref<Thread>,
}

impl EventRef {
    /// Create a new event.
    /// Sets the thread state
    /// to blocked.
    pub fn new(thread: Ref<Thread>) -> Result<Ref<Self>> {
        thread.set_state(State::Blocked);

        Ref::new(EventRef {
            thread,
        })
    }
    
    /// Trigger the event.
    pub fn trigger(self) -> Result<()> {
        self.thread.set_state(State::Ready);
        self.thread.resume()
    }
}
