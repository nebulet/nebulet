use super::dispatcher::Dispatch;
use alloc::boxed::Box;
use arch::context::ThreadContext;
use arch::cpu::Dpc;
use arch::cpu::Local;
use common::table::TableSlot;
use core::ptr;
use event::{Event, EventVariant};
use memory::sip::WasmStack;
use nabi::{Error, Result};
use object::Process;
use sync::atomic::{Atomic, Ordering};
use sync::mpsc::IntrusiveNode;

impl IntrusiveNode for Thread {
    #[inline]
    unsafe fn get_next(self: *mut Thread) -> *mut Thread {
        (*self).next_thread
    }

    #[inline]
    unsafe fn set_next(self: *mut Thread, next: *mut Thread) {
        (*self).next_thread = next;
    }

    #[inline]
    unsafe fn is_on_queue(self: *mut Thread) -> bool {
        !(*self).next_thread.is_null()
    }
}

/// The current state of a thread.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// The thread has not yet been started.
    Initial,
    /// The thread is ready to execute.
    Ready,
    /// The thread is currently executing.
    Running,
    /// The thread has been suspended and cannot be run right now.
    Suspended,
    /// The thread is blocked.
    Blocked,
    /// Ready to be killed.
    Killable,
    /// It's dead, Jim.
    Dead,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

/// Represents a thread.
pub struct Thread {
    pub ctx: ThreadContext,
    pub stack: WasmStack,

    exit_event: Event,

    func: *const (),

    parent: Option<Dispatch<Process>>,

    /// for intrusive queue
    next_thread: *mut Thread,

    /// process-local thread id
    local_id: TableSlot,

    state: Atomic<State>,
}

impl Thread {
    pub fn new<F>(stack_size: usize, f: F) -> Result<Box<Thread>>
    where
        F: FnOnce() + Send + Sync,
    {
        let stack = WasmStack::allocate(stack_size).ok_or(Error::NO_MEMORY)?;

        let exit_event = Event::new(EventVariant::Normal);

        Ok(Box::new(Thread {
            ctx: ThreadContext::new(stack.top(), common_thread_entry::<F>),
            stack,
            exit_event,
            func: Box::into_raw(Box::new(f)) as *const (),
            parent: None,
            next_thread: ptr::null_mut(),
            local_id: TableSlot::invalid(),
            state: Atomic::new(State::Initial),
        }))
    }

    pub fn new_with_parent<F>(
        parent: Dispatch<Process>,
        local_id: TableSlot,
        stack_size: usize,
        f: F,
    ) -> Result<Box<Thread>>
    where
        F: FnOnce() + Send + Sync,
    {
        let stack = WasmStack::allocate(stack_size).ok_or(Error::NO_MEMORY)?;

        let exit_event = Event::new(EventVariant::Normal);

        Ok(Box::new(Thread {
            ctx: ThreadContext::new(stack.top(), common_thread_entry::<F>),
            stack,
            exit_event,
            func: Box::into_raw(Box::new(f)) as *const (),
            parent: Some(parent),
            next_thread: ptr::null_mut(),
            local_id,
            state: Atomic::new(State::Initial),
        }))
    }

    pub fn start(&mut self) {
        let old_state = self
            .state
            .compare_and_swap(State::Initial, State::Ready, Ordering::SeqCst);

        debug_assert!(old_state == State::Initial);

        Local::schedule_thread(self);
    }

    pub fn state(&self) -> State {
        self.state.load(Ordering::Relaxed)
    }

    pub fn set_state(&self, state: State) {
        self.state.store(state, Ordering::Relaxed);
    }

    pub fn current<'a>() -> &'a mut Thread {
        unsafe { &mut *Local::current_thread() }
    }

    /// Yield the current thread.
    pub fn yield_now() {
        unsafe {
            Local::context_switch();
        }
    }

    pub fn parent(&self) -> Option<&Dispatch<Process>> {
        self.parent.as_ref()
    }

    pub fn resume(&self) {
        debug_assert!({
            let state = self.state();
            state == State::Blocked || state == State::Suspended
        });

        self.set_state(State::Ready);

        Local::schedule_thread(self as *const _ as *mut _);
    }

    pub fn join(self: Box<Self>) -> Result<()> {
        // cannot join with the current thread
        if &*self as *const _ == Thread::current() as *const _ {
            return Err(Error::INVALID_ARG);
        }

        let state = self.state();

        if state != State::Dead || state != State::Killable {
            self.exit_event.wait();
        }

        // At this point the thread denoted by `self` has died
        // so it's safe to let it drop.
        Ok(())
    }

    pub fn kill(self: Box<Self>) {
        if &*self as *const _ == Thread::current() as *const _ {
            return;
        }

        if !self.next_thread.is_null() {
            // the thread is on the runqueue
            self.set_state(State::Killable);
            // Don't drop the thread now the scheduler will take care of it.
            Box::into_raw(self);
        }
        // Otherwise, we can just let it drop.
    }

    // exit the current thread
    pub fn exit() {
        let current_thread = Thread::current();

        debug_assert!(current_thread.next_thread.is_null());

        current_thread.set_state(State::Dead);

        current_thread.exit_event.signal(false);

        if let Some(parent) = current_thread.parent() {
            let boxed_thread = {
                let mut thread_list = parent.thread_list().write();
                thread_list.free(current_thread.local_id).unwrap()
            };

            Dpc::cleanup_thread(Box::into_raw(boxed_thread));
        }

        unsafe {
            Local::context_switch();
        }

        unreachable!()
    }
}

extern "C" fn common_thread_entry<F>()
where
    F: FnOnce() + Send + Sync,
{
    let current_thread = Thread::current();

    let f = unsafe { Box::from_raw(current_thread.func as *mut F) };
    f();

    Thread::exit();

    unreachable!();
}
