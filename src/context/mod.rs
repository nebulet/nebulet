
#[path = "arch/x86_64.rs"]
pub mod arch;
pub mod context;
pub mod list;
pub mod memory;

pub use self::context::{Context, ContextId, State};
pub use self::list::ContextList;
pub use self::memory::Memory;

use core::sync::atomic::{AtomicUsize, Ordering};
use core::ops::DerefMut;
use alloc::{VecDeque, String};
use spin::RwLock;

/// Limit on number of contexts
pub const MAX_CONTEXTS: usize = (isize::max_value() as usize) - 1;

/// Initial stack size for contexts
pub const INITIAL_STACK_SIZE: usize = 1 * 1024; // 1 KB

lazy_static! {
    /// Scheduler instance
    pub static ref SCHEDULER: Scheduler = Scheduler::new();
}

/// The context scheduler.
/// The scheduler owns the list of contexts and
/// the list of contexts that are ready to be used.
/// Every field of the Scheduler struct is concurrent-safe.
pub struct Scheduler {
    current_id: AtomicUsize,
    context_table: RwLock<ContextList>,
    ready_list: RwLock<VecDeque<ContextId>>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            current_id: AtomicUsize::new(ContextId::KERNEL.into()),
            context_table: RwLock::new(ContextList::new()),
            ready_list: RwLock::new(VecDeque::new()),
        }
    }

    pub fn spawn(&self, name: String, f: extern "C" fn()) -> Result<ContextId, ()> {
        let mut context_list_lock = self.context_table.write();
        let mut new_context_id = context_list_lock.spawn(f, name)?;

        // set up the context
        let mut context = context_list_lock
            .get(new_context_id)
            .expect("Could not retrive the context that was just created?")
            .write();

        let mut ready_list_lock = self.ready_list.write();
        ready_list_lock.push_back(new_context_id);

        Ok(new_context_id)
    }

    pub fn current_id(&self) -> ContextId {
        ContextId::from(self.current_id.load(Ordering::SeqCst))
    }

    pub fn kill(&self, id: ContextId) {
        // scope the locks away from the context switch
        {
            let table_lock = self.context_table.read();
            let mut context_lock = table_lock
                .get(id)
                .expect("Cannot kill a non-existant context")
                .write();

            context_lock.set_state(State::Exited);
            context_lock.stack = None;
            context_lock.name = None;
            context_lock.kstack = None;
        }

        unsafe {
            self.switch();
        }
    }

    pub fn set_ready(&self, id: ContextId) {
        self.ready_list
            .write()
            .push_back(id);
    }

    /// Perform a context switch to a new context
    pub unsafe fn switch(&self) {
        // shortcut if the ready_list is empty
        if self.ready_list.read().is_empty() {
            return;
        }

        let mut prev_ptr: *mut Context = ::core::ptr::null_mut();
        let mut next_ptr: *mut Context = ::core::ptr::null_mut();

        // Seperate the locks away from the context switch
        {
            let table_lock = self.context_table.read();
            let mut ready_list_lock = self.ready_list.write();

            let current_id = self.current_id();

            let mut prev = table_lock
                .get(current_id)
                .expect("Could not retrieve previous context")
                .write();

            if prev.state == State::Current {
                prev.set_state(State::Ready);
                ready_list_lock.push_back(current_id);
            }

            if let Some(next_id) = ready_list_lock.pop_front() {
                if next_id != self.current_id() {
                    let mut next = table_lock
                        .get(next_id)
                        .expect("Could not retrieve new context")
                        .write();

                    next.set_state(State::Current);

                    self.current_id.store(next.id.into(), Ordering::SeqCst);

                    // save context pointers
                    prev_ptr = prev.deref_mut() as *mut Context;
                    next_ptr = next.deref_mut() as *mut Context;
                }
            }
        }

        if !next_ptr.is_null() {
            assert!(!prev_ptr.is_null());

            let prev = &mut *prev_ptr;
            let next = &mut *next_ptr;

            prev.context.switch_to(&mut next.context);
        }
    }
}