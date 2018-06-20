use nil::{Ref, HandleRef};
use object::{Thread, Process};
use task::State;
use sync::mpsc::Mpsc;
use nabi::{Result, Error};
use sync::atomic::{Atomic, Ordering};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EventState {
    Pending = 0,
    Done = 1,
}

#[derive(HandleRef)]
pub struct Event {
    queue: Mpsc<Ref<Thread>>,
    owner: Ref<Process>,
    state: Atomic<Result<EventState>>,
}

impl Event {
    /// Create a new event.
    /// The returned event can only
    /// be triggered by the process
    /// that created it.
    pub fn new() -> Event {
        Event {
            queue: Mpsc::new(),
            owner: Thread::current().parent().clone(),
            state: Atomic::new(Ok(EventState::Pending)),
        }
    }

    /// Returns `true` if the thread
    /// queue contains one or more threads.
    pub fn has_queued(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Wait on the event. This blocks the current thread.
    pub fn wait(&self) {
        if self.poll() != Ok(EventState::Pending) {
            return;
        }

        let current_thread = Thread::current();

        self.queue.push(current_thread.clone()); // this must be first
        current_thread.set_state(State::Blocked);

        Thread::yield_now();
    }

    pub fn poll(&self) -> Result<EventState> {
        self.state.load(Ordering::Acquire)
    }
    
    /// Trigger the event.
    /// This assures that only this thread is
    /// accessing this instance. Returns the
    /// number of threads that have been resumed.
    /// If a thread other than the owning thread
    /// tries to trigger the event, this will return `Error::ACCESS_DENIED`.
    pub fn trigger(&self) -> Result<usize> {
        if !Thread::current().parent().ptr_eq(&self.owner) {
            return Err(Error::ACCESS_DENIED);
        }

        let _ = self.state.compare_exchange(
            Ok(EventState::Pending),
            Ok(EventState::Done),
            Ordering::Release,
            Ordering::Relaxed
        ).map_err(|_| Error::ACCESS_DENIED)?;

        let mut count = 0;
        while let Some(thread) = self.queue.pop() {
            count += 1;
            thread.resume();
        }

        Ok(count)
    }

    pub fn rearm(&self) -> bool {
        self.state
            .compare_exchange(
                Ok(EventState::Done),
                Ok(EventState::Pending),
                Ordering::Release,
                Ordering::Relaxed
            )
            .is_ok()
    }
}
