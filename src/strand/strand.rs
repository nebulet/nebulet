use alloc::String;
use strand::Stack;
use arch::context::Context;
use arch::cpu;

use nabi::{Result};

/// The current state of a process.
#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Running,
    Suspended,
    Ready,
    Dead,
}

/// A single strand of execution.
#[derive(Debug)]
pub struct Strand {
    pub name: String,
    pub state: State,
    pub ctx: Context,
    pub stack: Stack,
    pub entry: extern fn(usize) -> i32,
    pub arg: usize,
    pub retcode: i32,
}

impl Strand {
    pub fn new<S: Into<String>>(name: S, entry: extern fn(usize) -> i32, arg: usize) -> Result<Strand> {
        let stack = Stack::new()?;
        let mut context = Context::new();
        context.rsp = stack.top() as usize;
        unsafe { context.push_stack(common_strand_entry as usize); }

        Ok(Strand {
            name: name.into(),
            state: State::Suspended,
            ctx: context,
            stack: stack,
            entry: entry,
            arg: arg,
            retcode: 0,
        })
    }

    pub fn resume(&mut self) -> Result<usize> {
        self.state = State::Ready;

        Ok(0)
    }
}

pub extern fn common_strand_entry() -> ! {
    let mut strand = cpu::strand::get();

    debug_assert!(strand.state == State::Suspended);

    // Execute the strand.
    let ret = (strand.entry)(strand.arg);

    // TODO: Exit the current strand.
    // This should never return.
    
    loop {}

    unreachable!();
}
