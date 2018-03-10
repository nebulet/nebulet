use super::{Thread, LockedThread, ThreadPriority, ThreadState};
use task;
use thread;
use time::{Instant, Duration};

use alloc::LinkedList;
use core::default::Default;
use core::{mem, ptr};
use core::cmp::min;
use core::sync::atomic::{AtomicU32, ATOMIC_U32_INIT, AtomicPtr, Ordering};

use spin::RwLock;

use macros::println;

#[derive(Debug)]
pub struct Scheduler {
    /// Array of thread lists.
    run_queues: [RwLock<LinkedList<LockedThread>>; ThreadPriority::NUM],
    /// Idle thread when no threads are left to run.
    idle_thread: LockedThread,
    /// Used to quickly find the highest priority level
    /// with threads to run.
    bitmap: AtomicU32,
}

impl Scheduler {
    /// Create a new scheduler instance
    /// This also creates a "null thread" that idles
    /// in the lowest priority.
    pub fn new() -> Scheduler {
        let idle_thread_lock = LockedThread::create("[idle]", task::thread::idle_thread_entry, 0, 32)
                .expect("Scheduler::new: Could not create the idle thread");

        {
            thread::set_current_thread(&mut *idle_thread_lock.write());
        }

        Scheduler {
            run_queues: Default::default(),
            idle_thread: idle_thread_lock,
            bitmap: ATOMIC_U32_INIT, // the lowest priority level contains a thread
        }
    }

    pub fn insert_thread_front(&self, thread: &LockedThread) {
        let priority_index = {
            let thread = thread.read();
            thread.priority.effective as usize 
        };
        
        let mut queue = self.run_queues[priority_index].write();

        self.bitmap.fetch_or(1 << priority_index, Ordering::Relaxed);

        queue.push_front(thread.clone());
    }

    pub fn insert_thread_back(&self, thread: &LockedThread) {
        let priority_index = {
            let thread = thread.read();
            thread.priority.effective as usize 
        };
        
        let mut queue = self.run_queues[priority_index].write();

        self.bitmap.fetch_or(1 << priority_index, Ordering::Relaxed);

        queue.push_back(thread.clone());
    }

    /// This retrieves the next thread that should be scheduled
    /// by popping a thread off the highest priority level
    /// that contains threads to run
    /// 
    /// # Warning:
    /// Currently, if the current thread is the same as the
    /// idle thread, `resched` will deadlock.
    fn get_top_thread(&self) -> LockedThread {
        let bitmap = self.bitmap.load(Ordering::Relaxed);
        if likely!(bitmap != 0) {
            let highest_queue = ThreadPriority::HIGHEST as usize - bitmap.leading_zeros() as usize
                - (mem::size_of_val(&self.bitmap) * 8 - ThreadPriority::NUM);

            debug_assert!(highest_queue < ThreadPriority::NUM);

            let mut queue = self.run_queues[highest_queue].write();

            let thread = queue.pop_front()
                .expect("Attempted to pop thread off empty list");

            if queue.is_empty() {
                self.bitmap.fetch_and(!(1 << highest_queue), Ordering::Relaxed);
            }

            thread
        } else {
            // No threads to run, return the idle thread
            // This returns an arced reference to the thread
            self.idle_thread.clone()
        }
    }

    pub fn resched(&self) {
        let mut old_thread_ptr: *mut Thread = ptr::null_mut();
        let mut new_thread_ptr: *mut Thread = ptr::null_mut();
        {
            let new_thread_lock = self.get_top_thread();
            {
                let mut new_thread = new_thread_lock.write();

                new_thread.state = ThreadState::Running;

                let mut old_thread = thread::get_current_thread();

                if &*new_thread as *const Thread as usize == &*old_thread as *const Thread as usize {
                    // if it's the same thread, return
                    return;
                }

                let now = Instant::now();

                // handle thread quantums
                debug_assert!(now >= old_thread.last_started_running);
                let old_runtime = now - old_thread.last_started_running;
                old_thread.runtime += old_runtime;
                old_thread.remaining_time_slice -= min(old_runtime, old_thread.remaining_time_slice);

                // setup quantum for the new thread if it was consumed
                if new_thread.remaining_time_slice == Duration::from_secs(0) {
                    new_thread.remaining_time_slice = Thread::INTIAL_TIME_SLICE;
                }

                new_thread.last_started_running = now;

                // TODO: Set up onshot timer to handle preemption

                // save current thread
                thread::set_current_thread(&mut *new_thread);

                old_thread_ptr = &mut *old_thread;
                new_thread_ptr = &mut *new_thread;
            }
        }

        debug_assert!(!old_thread_ptr.is_null() && !new_thread_ptr.is_null());

        // context switch
        unsafe {
            thread::context_switch(&mut *old_thread_ptr, &mut *new_thread_ptr);
        }
    }
}