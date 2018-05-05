use alloc::VecDeque;
use super::thread_entry::ThreadEntry;
use super::thread::{Thread, State};
use super::ThreadTable;
use arch::lock::Spinlock;

struct SchedulerInner {
    ready_queue: VecDeque<ThreadEntry>,
    current_thread: Option<ThreadEntry>,
}

pub struct Scheduler {
    inner: Spinlock<SchedulerInner>,
}

impl Scheduler {
    pub fn new(idle_thread: ThreadEntry) -> Scheduler {
        Scheduler {
            inner: Spinlock::new(SchedulerInner {
                ready_queue: VecDeque::new(),
                current_thread: Some(idle_thread),
            }),
        }
    }
    
    /// Adds a thread index to the end of the queue.
    pub fn push(&self, entry: ThreadEntry) {
        self.inner
            .lock()
            .ready_queue
            .push_back(entry);
    }

    /// Switch to the next thread.
    pub fn switch(&self) {
        if let (Some(next_thread_entry), Some(current_thread_entry)) = {
            let mut inner_guard = self.inner.lock();

            let next_thread = inner_guard
                .ready_queue
                .pop_front();
            let current_thread = inner_guard
                .current_thread;

            (next_thread, current_thread)
        } {
            if next_thread_entry == current_thread_entry {
                return;
            }

            // set the current thread
            {
                self.inner.lock().current_thread = Some(next_thread_entry);
            }
            
            let (mut current_thread, mut next_thread) = {
                let mut thread_table_guard = ThreadTable::lock();
                
                let mut current_thread_ptr = thread_table_guard
                    .get_mut(current_thread_entry.id())
                    .unwrap() as *mut Thread;
                let next_thread_ptr = thread_table_guard
                    .get_mut(next_thread_entry.id())
                    .unwrap() as *mut Thread;
                
                unsafe { (&mut *current_thread_ptr, &mut *next_thread_ptr) }
            };

            assert!(next_thread.state() == State::Ready);

            current_thread.set_state(State::Ready);
            self.push(current_thread_entry);

            next_thread.set_state(State::Running);

            // So, at this point, the table_guard should be released, and we should also have
            // references to both the current thread and the next thread.
            // This is wildly unsafe and is probably broken.
            unsafe {
                current_thread.switch_to(next_thread);
            }
        }
    }
}