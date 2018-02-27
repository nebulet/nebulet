
#[path = "arch/x86_64.rs"]
pub mod arch;
pub mod context;
pub mod list;
pub mod memory;

pub use self::context::{Context, ContextId, State};
pub use self::list::ContextList;
pub use self::memory::Memory;

use core::sync::atomic::AtomicUsize;
use alloc::{VecDeque, String};
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard, Once};

/// Limit on number of contexts
pub const MAX_CONTEXTS: usize = (isize::max_value() as usize) - 1;

/// Initial stack size for contexts
pub const INITIAL_STACK_SIZE: usize = 1 * 1024; // 1 KB

/// Contexts list
static CONTEXTS: Once<RwLock<ContextList>> = Once::new();

/// Initialize contexts
fn init_contexts() -> RwLock<ContextList> {
    RwLock::new(ContextList::new())
}

/// Get the global context list
pub fn contexts() -> RwLockReadGuard<'static, ContextList> {
    CONTEXTS.call_once(init_contexts).read()
}

/// Get a mutable global context list
pub fn contexts_mut() -> RwLockWriteGuard<'static, ContextList> {
    CONTEXTS.call_once(init_contexts).write()
}

/// The context scheduler
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

    pub fn spawn(&mut self, f: extern "C" fn(), name: String) -> Result<ContextId, ()> {
        let mut context_list_lock = self.context_table.write();
        let mut new_context_id = context_list_lock.spawn(f, name)?;
        let mut ready_list_lock = self.ready_list.write();
        ready_list_lock.push_back(new_context_id);

        Ok(new_context_id)
    }

    pub fn switch(&mut self) {
        // shortcut if the ready_list is empty
        if self.ready_list.read().is_empty() {
            return;
        }
    }
}