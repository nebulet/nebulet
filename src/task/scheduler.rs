use alloc::VecDeque;
use super::thread::{State, ThreadRef};
use arch::lock::IrqSpinlock;

struct SchedulerInner {
    ready_queue: VecDeque<ThreadRef>,
    current_thread: ThreadRef,
}

pub struct Scheduler {
    inner: IrqSpinlock<SchedulerInner>,
    idle_thread: ThreadRef,
}

impl Scheduler {
    pub fn new(kernel_thread: ThreadRef, idle_thread: ThreadRef) -> Scheduler {
        Scheduler {
            inner: IrqSpinlock::new(SchedulerInner {
                ready_queue: VecDeque::new(),
                current_thread: kernel_thread,
            }),
            idle_thread,
        }
    }
    
    /// Adds a thread index to the end of the queue.
    pub fn push(&self, thread_ref: ThreadRef) {
        self.inner
            .lock()
            .ready_queue
            .push_back(thread_ref);
    }

    pub fn switch(&self) {
        // These will either get dropped
        // or voluntarily released before
        // switching contexts.
        let mut inner = self.inner.lock();

        let mut current_thread = inner.current_thread;

        // Either switch to the next thread in the queue or the idle thread.
        let mut next_thread = if let Some(next_thread) = inner.ready_queue.pop_front() {
            next_thread
        } else {
            if current_thread.state == State::Running {
                current_thread
            } else {
                self.idle_thread
            }
        };

        if next_thread == current_thread {
            return;
        }

        inner.current_thread = next_thread;

        debug_assert!(next_thread.state == State::Ready);

        if current_thread.state == State::Running && current_thread != self.idle_thread {
            current_thread.state = State::Ready;
            inner.ready_queue.push_back(current_thread);
        }

        next_thread.state = State::Running;

        // Release the lock so we don't deadlock
        inner.release();

        unsafe {
            current_thread.swap(&next_thread);
        }
    }
}