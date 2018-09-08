use arch::cpu::Local;
use arch::cpu::{Dpc, IrqController};
use object::thread::{State, Thread};
use sync::mpsc::{IntrusiveMpsc, IntrusiveNode};

/// The Scheduler schedules threads to be run.
/// Currently, it's a simple round-robin.
pub struct Scheduler {
    thread_queue: IntrusiveMpsc<Thread>,
    idle_thread: *mut Thread,
}

impl Scheduler {
    pub fn new(idle_thread: *mut Thread) -> Scheduler {
        Scheduler {
            thread_queue: IntrusiveMpsc::new(),
            idle_thread,
        }
    }

    pub fn schedule_thread(&self, thread: *mut Thread) {
        unsafe {
            self.thread_queue.push(thread);
        }
    }

    pub unsafe fn switch(&self) {
        // disable irqs while in the scheduler.
        IrqController::disable();

        let current_thread = Thread::current();

        let next_thread = loop {
            if let Some(next_thread) = self.thread_queue.pop() {
                debug_assert!(!next_thread.is_on_queue());

                let state = (*next_thread).state();
                if state == State::Ready {
                    break next_thread;
                } else if state == State::Killable {
                    // the scheduler should kill this thread
                    (*next_thread).set_state(State::Dead);
                    Dpc::cleanup_thread(next_thread);
                }
            } else {
                // no threads in the run queue
                if (*current_thread).state() == State::Running {
                    // One thread running in this scheduler,
                    // so no need to context switch.
                    return;
                } else {
                    break self.idle_thread;
                }
            }
        };

        if current_thread.state() == State::Running {
            current_thread.set_state(State::Ready);
            if current_thread as *const _ != self.idle_thread as *const _ {
                self.thread_queue.push(current_thread);
            }
        }

        debug_assert!((*next_thread).state() == State::Ready);

        (*next_thread).set_state(State::Running);

        Local::set_current_thread(next_thread);

        current_thread.ctx.swap(&(*next_thread).ctx);

        IrqController::enable();
    }
}
