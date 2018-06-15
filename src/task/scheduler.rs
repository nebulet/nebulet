use super::thread::{Thread as TaskThread, State};
use arch::cpu::Local;
use common::mpsc::{Mpsc, Reciever, Sender};
use object::Thread;
use nil::Ref;

/// The Scheduler schedules threads to be run.
/// Currently, it's a simple, round-robin.
pub struct Scheduler {
    thread_tx: Sender<Ref<Thread>>,
    thread_rx: Reciever<Ref<Thread>>,
    idle_thread: Ref<Thread>,
}

impl Scheduler {
    pub fn new(idle_thread: Ref<Thread>) -> Scheduler {
        let (thread_tx, thread_rx) = Mpsc::new();
        Scheduler {
            thread_tx,
            thread_rx,
            idle_thread,
        }
    }

    pub fn thread_sender(&self) -> Sender<Ref<Thread>> {
        self.thread_tx.clone()
    }

    pub unsafe fn switch(&self) {
        let current_thread = Local::current_thread();

        let next_thread = if let Some(next_thread) = self.thread_rx.recv() {
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
            self.thread_tx.send(current_thread.clone());
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
