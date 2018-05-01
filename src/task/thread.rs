use super::stack::Stack;
use arch::context::Context;

use nabi::{Result};

/// The current state of a process.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// The thread is currently executing.
    Running,
    /// This thread is not currently running, but it's ready to execute.
    Ready,
    /// The thread has been preempted and cannot be run right now.
    Preempted,
    /// It's dead, Jim.
    Dead,
}

/// A single thread of execution.
#[derive(Debug)]
pub struct Thread {
    state: State,
    ctx: Context,
    stack: Stack,
    entry: extern fn(),
}

impl Thread {
    pub fn new(stack_size: usize, entry: extern fn()) -> Result<Thread> {
        let stack = Stack::with_size(stack_size)?;
        let ctx = Context::init(stack.top(), entry);

        Ok(Thread {
            state: State::Ready,
            ctx,
            stack,
            entry,
        })
    }

    pub unsafe fn switch_to(&mut self, other: &Thread) {
        self.ctx.switch_to(&other.ctx);
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }
}