use nil::{Ref, KernelRef};
use object::Thread;
use task::State;
use nabi::{Result};

#[derive(KernelRef)]
pub struct Event {
    thread: Ref<Thread>,
}

impl Event {
    /// Create a new event.
    /// Sets the thread state
    /// to blocked.
    pub fn new(thread: Ref<Thread>) -> Event {
        thread.set_state(State::Blocked);

        Event {
            thread,
        }
    }
    
    /// Trigger the event.
    pub fn trigger(self) -> Result<()> {
        self.thread.set_state(State::Ready);
        self.thread.resume()
    }
}
