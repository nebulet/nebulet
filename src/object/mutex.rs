use alloc::VecDeque;
use nil::{Ref, KernelRef};
use core::sync::atomic::{AtomicIsize, Ordering};
use arch::lock::Spinlock;
use arch::cpu::Local;
use object::ThreadRef;
use task::State;

#[derive(KernelRef)]
pub struct Mutex {
    wait_queue: Spinlock<VecDeque<Ref<ThreadRef>>>,
    counter: AtomicIsize,
}

impl Mutex {
    pub fn new() -> Mutex {
        Mutex {
            wait_queue: Spinlock::new(VecDeque::new()),
            counter: AtomicIsize::new(1),
        }
    }

    /// Acquire the mutex.
    pub fn acquire(&self) {
        let old = self.counter.fetch_sub(1, Ordering::SeqCst);

        if old != 1 {
            // We don't have the lock, so enqueue
            // ourselves in the wait queue and block.
            let current_thread = Local::current_thread();
            current_thread.set_state(State::Blocked);
            let mut wait_queue = self.wait_queue.lock();
            wait_queue.push_back(current_thread);
            
            unsafe {
                Local::context_switch();
            }
        }
    }

    /// Release the mutex.
    pub fn release(&self) {
        let old = self.counter.swap(1, Ordering::SeqCst);
        if old != 0 {
            // There are threads waiting on this lock
            let mut wait_queue = self.wait_queue.lock();

            if let Some(thread) = wait_queue.pop_front() {
                thread.set_state(State::Ready);
                Local::schedule_thread(thread);
            } else {
                unreachable!()
            }
        }
    }
}
