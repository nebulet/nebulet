use nil::{Ref, HandleRef};
use object::Thread;
use task::State;
use sync::mpsc::Mpsc;
use nabi::{Result, Error};

#[derive(HandleRef)]
pub struct Event {
    queue: Mpsc<Ref<Thread>>,
    owner: Ref<Thread>,
}

impl Event {
    /// Create a new event.
    /// The returned event can only
    /// be triggered by the thread
    /// that created it.
    pub fn new() -> Event {
        Event {
            queue: Mpsc::new(),
            owner: Thread::current(),
        }
    }

    /// Wait on the event. This blocks the current thread.
    pub fn wait(&self) {
        let current_thread = Thread::current();
        current_thread.set_state(State::Blocked);
        self.queue.push(current_thread);
        Thread::yield_now();
    }
    
    /// Trigger the event.
    /// This assures that only this thread is
    /// accessing this instance. Returns the
    /// number of threads that have been resumed.
    /// If a thread other than the owning thread
    /// tries to trigger the event, this will return `Error::ACCESS_DENIED`.
    pub fn trigger(&self) -> Result<usize> {
        let current_thread = Thread::current();
        if !current_thread.ptr_eq(&self.owner) {
            return Err(Error::ACCESS_DENIED);
        }

        let mut count = 0;
        while let Some(thread) = self.queue.pop() {
            count += 1;
            thread.resume();
        }
        Ok(count)
    }
}
