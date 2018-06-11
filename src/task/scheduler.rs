use alloc::VecDeque;
use super::thread::{Thread, State};
use arch::cpu::Local;
use arch::lock::IrqSpinlock;
use object::ThreadRef;
use nil::Ref;

/// The Scheduler schedules threads to be run.
/// Currently, it's a simple, round-robin.
pub struct Scheduler {
    ready_queue: IrqSpinlock<VecDeque<Ref<ThreadRef>>>,
    idle_thread: Ref<ThreadRef>,
}

impl Scheduler {
    pub fn new(idle_thread: Ref<ThreadRef>) -> Scheduler {
        Scheduler {
            ready_queue: IrqSpinlock::new(VecDeque::new()),
            idle_thread,
        }
    }
    
    /// Adds a thread index to the end of the queue.
    pub fn push(&self, thread: Ref<ThreadRef>) {
        self.ready_queue
            .lock()
            .push_back(thread);
    }

    pub unsafe fn switch(&self) {
        let mut ready_queue = self.ready_queue.lock();
        let current_thread = Local::current_thread();

        let next_thread = if let Some(next_thread) = ready_queue.pop_front() {
            next_thread
        } else {
            if current_thread.state() == State::Running {
                current_thread.clone()
            } else {
                self.idle_thread.clone()
            }
        };

        if next_thread.ptr_eq(&current_thread) {
            return;
        }

        debug_assert!(next_thread.state() == State::Ready);

        if current_thread.state() == State::Running && !current_thread.ptr_eq(&self.idle_thread) {
            current_thread.set_state(State::Ready);
            ready_queue.push_back(current_thread.clone());
        }

        next_thread.set_state(State::Running);

        Local::set_current_thread(next_thread.clone());

        let (current_thread_inner, next_thread_inner) = {
            (
                &mut *(&mut *current_thread.inner().lock() as *mut Thread),
                &*(&*next_thread.inner().lock() as *const Thread),
            )
        };

        // Release the lock so we don't deadlock
        ready_queue.release();

        current_thread_inner.swap(next_thread_inner);
    }
}
