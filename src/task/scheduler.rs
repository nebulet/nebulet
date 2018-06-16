use super::thread::{Thread as TaskThread, State};
use arch::cpu::Local;
use sync::mpsc::Mpsc;
use object::Thread;
use nil::Ref;

/// The Scheduler schedules threads to be run.
/// Currently, it's a simple, round-robin.
pub struct Scheduler {
    thread_queue: Mpsc<Ref<Thread>>,
    idle_thread: Ref<Thread>,
}

impl Scheduler {
    pub fn new(idle_thread: Ref<Thread>) -> Scheduler {
        let thread_queue = Mpsc::new();
        Scheduler {
            thread_queue,
            idle_thread,
        }
    }

    #[inline]
    pub fn schedule_thread(&self, thread: Ref<Thread>) {
        self.thread_queue.push(thread);
    }

    pub unsafe fn switch(&self) {
        let current_thread = Local::current_thread();

        let next_thread = if let Some(next_thread) = self.thread_queue.pop() {
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
            self.thread_queue.push(current_thread.clone());
        }

        next_thread.set_state(State::Running);

        Local::set_current_thread(next_thread.clone());

        let (current_thread_inner, next_thread_inner) = {
            (
                &mut *(&mut *current_thread.inner().lock() as *mut TaskThread),
                &*(&*next_thread.inner().lock() as *const TaskThread),
            )
        };

        current_thread_inner.swap(next_thread_inner);
    }
}
