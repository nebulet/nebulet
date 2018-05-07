use alloc::VecDeque;
use super::thread_entry::ThreadEntry;
use super::thread::{Thread, State};
use super::ThreadTable;
use arch::lock::IrqSpinlock;

struct SchedulerInner {
    ready_queue: VecDeque<ThreadEntry>,
    current_thread: ThreadEntry,
}

pub struct Scheduler {
    inner: IrqSpinlock<SchedulerInner>,
    idle_thread: ThreadEntry,
}

impl Scheduler {
    pub fn new(kernel_thread: ThreadEntry, idle_thread: ThreadEntry) -> Scheduler {
        Scheduler {
            inner: IrqSpinlock::new(SchedulerInner {
                ready_queue: VecDeque::new(),
                current_thread: kernel_thread,
            }),
            idle_thread,
        }
    }
    
    /// Adds a thread index to the end of the queue.
    pub fn push(&self, entry: ThreadEntry) {
        self.inner
            .lock()
            .ready_queue
            .push_back(entry);
    }

    pub fn switch(&self) {
        // These will either get dropped
        // or voluntarily released before
        // switching contexts.
        let mut inner = self.inner.lock();
        let mut thread_table = ThreadTable::lock();

        let current_entry = inner.current_thread;

        let current_thread = unsafe {
            &mut *((&mut thread_table[current_entry.id()] as *mut Thread))
        };

        // Either switch to the next thread in the queue or the idle thread.
        let next_entry = if let Some(next_entry) = inner.ready_queue.pop_front() {
            next_entry
        } else {
            if current_thread.state() == State::Running {
                current_entry
            } else {
                self.idle_thread
            }
        };

        if next_entry == current_entry {
            return;
        }

        inner.current_thread = next_entry;

        // Get references to the current and
        // next thread, but with artificially
        // extended lifetimes.
        let next_thread = unsafe {
            &mut *((&mut thread_table[next_entry.id()] as *mut Thread))
        };

        debug_assert!(next_thread.state() == State::Ready);

        if current_thread.state() == State::Running && current_entry != self.idle_thread {
            current_thread.set_state(State::Ready);
            inner.ready_queue.push_back(current_entry);
        }

        next_thread.set_state(State::Running);

        // println!("Switching from thread[{}] to thread[{}]", current_thread.name, next_thread.name);

        // Release the locks so we don't deadlock
        inner.release();
        thread_table.release();

        unsafe {
            current_thread.swap(next_thread);
        }
    }
}