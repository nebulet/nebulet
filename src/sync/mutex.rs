use nil::Ref;
use core::cell::Cell;
use core::ptr;
use arch::cpu::Local;
use sync::mpsc::Mpsc;
use sync::atomic::*;
use object::Thread;
// use task::State;

pub struct WaitRecord {
    thread: Option<Ref<Thread>>,
}

impl WaitRecord {
    pub fn new(thread: Ref<Thread>) -> WaitRecord {
        WaitRecord {
            thread: Some(thread),
        }
    }

    pub fn wait(&mut self) {
        use task::State;
        if let Some(ref thread) = self.thread {
            thread.set_state(State::Blocked);
        }
    }

    pub fn wake(&mut self) {
        use task::State;
        use arch::cpu::Local;
        if let Some(thread) = self.thread.take() {
            thread.set_state(State::Ready);
            Local::schedule_thread(thread);
        }
    }

    pub fn thread(&self) -> Option<*const Thread> {
        self.thread.as_ref().map(|thread| &**thread as *const Thread)
    }
}

pub struct Mutex {
    count: Atomic<isize>,
    owner: Atomic<*const Thread>,
    waitqueue: Mpsc<*mut WaitRecord>,
    handoff: Atomic<u32>,
    depth: Cell<u32>,
    sequence: Cell<u32>,
}

impl Mutex {
    pub const fn new() -> Mutex {
        Mutex {
            count: Atomic::new(0),
            owner: Atomic::new(ptr::null_mut()),
            waitqueue: Mpsc::new(),
            handoff: Atomic::new(0),
            depth: Cell::new(0),
            sequence: Cell::new(0),
        }
    }

    pub fn acquire(&self) {
        let current_thread = Local::current_thread();

        if self.count.fetch_add(1, Ordering::Acquire) == 0 {
            // The lock is uncontented, so we can immediately acquire it
            self.owner.store(&*current_thread as *const _, Ordering::Relaxed);
            self.depth.set(1);
            return;
        }

        // If we're here, the mutex was already locked,
        // but it's possible the lock holder is us.
        if self.owner.load(Ordering::Relaxed) == &*current_thread as *const _ {
            self.count.fetch_sub(1, Ordering::Relaxed);

            let mut depth = self.depth.get();
            depth += 1;
            self.depth.set(depth);

            return;
        }

        // If we're still here, the lock is owned by a different thread
        // So, put the current thread in the wait queue.
        // the wait_record is on the stack, so we have to make sure
        // that it gets popped out of the waitqueue before returning.
        let mut waiter = WaitRecord::new(current_thread.clone());
        self.waitqueue.push(&mut waiter);

        let old_handoff = self.handoff.load(Ordering::SeqCst);
        if old_handoff != 0 {
            if !self.waitqueue.is_empty() {
                if self.handoff.compare_exchange(old_handoff, 0, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                    // We can be sure that the waitqueue is not empty
                    let other_wr = unsafe { &mut *self.waitqueue.pop().unwrap() };
                    if other_wr.thread() != Some(&*current_thread as *const Thread) {
                        // At this point, waiter.thread() must be Some(_), otherwise
                        // it means someone has already woken us up, breaking the
                        // handoff protocol.
                        assert!(waiter.thread().is_some());
                        other_wr.wake();
                    } else {
                        // got the lock ourselves
                        assert!(other_wr as *const _ == &mut waiter as *const _);
                        self.owner.store(&*current_thread as *const _, Ordering::Relaxed);
                        self.depth.set(1);
                        return;
                    }
                }
            }
        }

        waiter.wait();
        self.owner.store(&*current_thread as *const _, Ordering::Relaxed);
        self.depth.set(1);
    }

    pub fn release(&self) {
        // we assume `release()` is only called when this thread
        // is holding the lock. We'll check just to make sure.
        let current_thread = Local::current_thread();
        if self.owner.load(Ordering::Relaxed) == &*current_thread as *const Thread {
            return;
        }

        let mut depth = self.depth.get();
        assert!(depth != 0);
        depth -= 1;
        self.depth.set(depth);
        if depth != 0 {
            // recursively locked
            return;
        }

        // When we return from `release()`, we will no longer be holding
        // the lock. We have to clear the current owner.
        self.owner.store(ptr::null_mut(), Ordering::Relaxed);

        // If there is no waiting `acquire()`, we're done.
        if self.count.fetch_sub(1, Ordering::Release) == 1 {
            return;
        }

        // Otherwise, there is at least one concurrent lock.
        loop {
            if let Some(other) = self.waitqueue.pop() {
                let other: &mut WaitRecord = unsafe { &mut*other };
                assert!(other.thread() != Some(&*current_thread as *const Thread));
                other.wake();
                break;
            } else {
                // Some concurrent `acquire()` is in progress, but hasn't yet put itself
                // into the wait queue.
                let mut sequence = self.sequence.get() + 1;
                if sequence == 1 {
                    sequence += 1;
                }
                self.handoff.store(sequence, Ordering::SeqCst);

                // if the waitqueue, the concurrent `acquire()` is before
                // adding itself, and therefore will definitely find our handoff
                // later.
                if self.waitqueue.is_empty() {
                    break;
                }
                // A thread already appeared on the wait queue, let's try
                // to take the handoff ourselves, and awaken it. If someone
                // else already took the handoff, they're responsible now.
                if self.handoff.compare_exchange(sequence, 0, Ordering::SeqCst, Ordering::Relaxed).is_err() {
                    break;
                }
            }
        }
    }
}
