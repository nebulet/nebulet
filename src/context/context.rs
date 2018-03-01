use alloc::Vec;
use alloc::boxed::Box;
use alloc::arc::Arc;

use context::{self, arch, Memory};
use common::Encapsulate;
use macros::println;

/// The status of a Context
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
    Ready,
    Current,
    Suspended,
    Exited,
}

/// Context priority
pub type Priority = Encapsulate<usize>;

/// Context Id
pub type ContextId = Encapsulate<usize>;

impl ContextId {
    /// Kernel Context
    pub const KERNEL: ContextId = ContextId::from(0);
}

/// A Context, which identifies a lightweight Context or thread
#[derive(Debug)]
pub struct Context {
    pub id: ContextId,
    pub state: State,
    pub context: arch::Context,
    pub priority: Priority,
    pub stack: Option<Memory>,
    pub name: Option<Arc<Box<str>>>,
    pub kstack: Option<Vec<u8>>,
}

impl Context {
    pub fn new(id: ContextId) -> Context {
        Context {
            id: id,
            state: State::Suspended,
            context: arch::Context::new(),
            priority: Priority::from(0),
            stack: None,
            name: None,
            kstack: None,
        }
    }

    /// Set the state of the Context
    pub fn set_state(&mut self, new: State) {
        self.state = new;
    }

    /// Set the stack pointer
    pub fn set_stack(&mut self, addr: usize) {
        self.context.set_stack(addr);
    }
}

/// A finishing Context returns to this function
#[naked]
pub unsafe extern "C" fn context_return() {
    // For now, just print that a Context returned
    println!("A Context returned!");

    let scheduler = &context::SCHEDULER;
    let current_id = scheduler.current_id();
    scheduler.kill(current_id);
}