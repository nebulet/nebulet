use object::thread::{Thread, State};
use sync::spsc::IntrusiveSpsc;
use arch::lock::Spinlock;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum EventVariant {
    /// Once the event is notified, let wake up one thread and then denotify.
    AutoUnsignal,
    /// Once the event is notified, let all threads through until manually denotified.
    Normal,
}

struct EventInner {
    // the thread will either be in 
    // a wait queue or the scheduler run queue.
    queue: IntrusiveSpsc<Thread>,
    notified: bool,
    variant: EventVariant,
}

pub struct Event {
    inner: Spinlock<EventInner>,
}

impl Event {
    /// Create a new event.
    /// The returned event can only
    /// be triggered by the process
    /// that created it.
    pub fn new(variant: EventVariant) -> Event {
        Event {
            inner: Spinlock::new(EventInner {
                queue: IntrusiveSpsc::new(),
                notified: false,
                variant,
            }),
        }
    }

    /// Returns `true` if the thread
    /// queue contains one or more threads.
    pub fn has_queued(&self) -> bool {
        !self.inner.lock().queue.is_empty()
    }

    /// Wait on the event. This blocks the current thread.
    pub fn wait(&self) {
        let current_thread = Thread::current();

        let mut inner = self.inner.lock();

        if inner.notified {
            if inner.variant == EventVariant::AutoUnsignal {
                inner.notified = false;
            }
        } else {
            // unnotified, block here
            unsafe { inner.queue.push(current_thread); }
            current_thread.set_state(State::Blocked);
            drop(inner);
            Thread::yield_now();
        }
    }

    /// Trigger the event.
    /// This assures that only this thread is
    /// accessing this instance. Returns the
    /// number of threads that have been resumed.
    pub fn signal(&self, reschedule: bool) -> usize {
        let mut inner = self.inner.lock();

        let mut wake_count = 0;
        
        if !inner.notified {
            if inner.variant == EventVariant::AutoUnsignal {
                unsafe {
                    if let Some(thread) = inner.queue.pop() {
                        (*thread).resume();
                        inner.notified = true;
                        wake_count = 1;
                    }
                }
            } else {
                inner.notified = true;
                unsafe {
                    while let Some(thread) = inner.queue.pop() {
                        (*thread).resume();
                        wake_count += 1;
                    }
                }
            }
        }

        drop(inner);

        if reschedule {
            Thread::yield_now();
        }

        wake_count
    }

    pub fn unsignal(&self) {
        self.inner.lock().notified = false;
    }
}
