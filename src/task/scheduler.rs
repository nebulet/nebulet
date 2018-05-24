use alloc::VecDeque;
use super::thread::{Thread, State};
use arch::cpu::Local;
use arch::lock::IrqSpinlock;

struct SchedulerInner {
    ready_queue: VecDeque<*mut Thread>,
    current_thread: *mut Thread,
}

pub struct Scheduler {
    inner: IrqSpinlock<SchedulerInner>,
    idle_thread: *mut Thread,
}

impl Scheduler {
    pub fn new(kernel_thread: *mut Thread, idle_thread: *mut Thread) -> Scheduler {
        Scheduler {
            inner: IrqSpinlock::new(SchedulerInner {
                ready_queue: VecDeque::new(),
                current_thread: kernel_thread,
            }),
            idle_thread,
        }
    }
    
    /// Adds a thread index to the end of the queue.
    pub fn push(&self, thread_ref: *mut Thread) {
        self.inner
            .lock()
            .ready_queue
            .push_back(thread_ref);
    }

    pub unsafe fn switch(&self) {
        // These will either get dropped
        // or voluntarily released before
        // switching contexts.
        let mut inner = self.inner.lock();

        let current_thread = &mut *inner.current_thread;

        // Either switch to the next thread in the queue or the idle thread.
        let next_thread = if let Some(next_thread) = inner.ready_queue.pop_front() {
            &mut *next_thread
        } else {
            if current_thread.state == State::Running {
                &mut *inner.current_thread
            } else {
                &mut *self.idle_thread
            }
        };

        if next_thread as *const _ as usize == current_thread as *const _ as usize {
            return;
        }

        inner.current_thread = next_thread;

        debug_assert!(next_thread.state == State::Ready);

        if current_thread.state == State::Running
            && current_thread as *const _ as usize != self.idle_thread as *const _ as usize
        {
            current_thread.state = State::Ready;
            inner.ready_queue.push_back(current_thread as *mut _);
        }

        next_thread.state = State::Running;

        Local::set_current_thread(next_thread.into());

        // Release the lock so we don't deadlock
        inner.release();

        current_thread.swap(&next_thread);
    }
}
