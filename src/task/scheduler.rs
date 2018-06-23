use super::thread::State;
use arch::cpu::Local;
use sync::mpsc::Mpsc;
use object::Thread;

/// The Scheduler schedules threads to be run.
/// Currently, it's a simple round-robin.
pub struct Scheduler {
    thread_queue: Mpsc<*const Thread>,
    idle_thread: *const Thread,
}

impl Scheduler {
    pub fn new(idle_thread: *const Thread) -> Scheduler {
        let thread_queue = Mpsc::new();
        Scheduler {
            thread_queue,
            idle_thread,
        }
    }

    pub fn schedule_thread(&self, thread: *const Thread) {
        self.thread_queue.push(thread);
    }

    pub unsafe fn switch(&self) {
        let current_thread = Local::current_thread();

        let next_thread = loop {
            if let Some(next_thread) = self.thread_queue.pop() {
                let state = (*next_thread).state();
                if state == State::Running {
                    break next_thread;
                }
            } else {
                if (*current_thread).state() == State::Running {
                    break current_thread;
                } else {
                    break self.idle_thread;
                }
            }
        };

        debug_assert!((*next_thread).state() == State::Running);

        if next_thread == current_thread {
            return;
        }

        if current_thread != self.idle_thread {
            self.thread_queue.push(current_thread);
        }

        Local::set_current_thread(next_thread);

        // println!("current_thread stacktop: {:p}", (*(*current_thread).inner().stack.get()).top());
        // println!("next_thread stacktop: {:p}", (*(*next_thread).inner().stack.get()).top());

        (*(*current_thread).inner()).swap(&*(*next_thread).inner());
    }
}
