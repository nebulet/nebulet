use alloc::String;
use strand::Stack;
use arch::context::Context;
use arch::cpu;

use nabi::{Result};

/// The current state of a process.
#[derive(Debug, PartialEq, Eq)]
pub enum State {
    /// The Strand is currently executing.
    Running,
    /// This Strand is not currently running, but it's ready to execute.
    Ready,
    /// The Strand has been suspended.
    Suspended,
    /// It's dead, Jim.
    Dead,
}

/// A single strand of execution.
#[derive(Debug)]
pub struct Strand {
    pub name: String,
    pub state: State,
    pub ctx: Context,
    pub stack: Stack,
    pub entry: extern fn(),
}

impl Strand {
    pub fn new<S: Into<String>>(name: S, entry: extern fn()) -> Result<Strand> {
        let stack = Stack::new()?;
        let mut ctx = Context::new();
        ctx.rsp = stack.top() as usize;
        unsafe { ctx.push_stack(common_strand_entry as usize); }

        Ok(Strand {
            name: name.into(),
            state: State::Suspended,
            ctx,
            stack,
            entry,
        })
    }

    pub fn resume(&mut self) -> Result<usize> {
        self.state = State::Ready;



        Ok(0)
    }
}

pub extern fn common_strand_entry() -> ! {
    let strand = cpu::strand::get();

    debug_assert!(strand.state == State::Suspended);

    // Execute the strand.
    (strand.entry)();

    // TODO: Exit the current strand.
    // This should never return.

    loop {}

    unreachable!();
}
