use context::arch;
use common::Encapsulate;
use macros::println;

/// The status of a process
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
    Ready,
    Current,
    Suspended,
    Exited,
}

/// Process priority
pub type Priority = Encapsulate<usize>;
/// Process Id
pub type ProcessId = Encapsulate<usize>;
impl ProcessId {
    /// Kernel process
    pub const KERNEL = ProcessId::from(0);
}

/// PID
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessId(usize);

/// A process, which identifies a lightweight process or thread
#[derive(Debug)]
pub struct Process {
    pub id: ProcessId,
    pub state: State,
    pub context: arch::Context,
    pub priority: Priority,
}

impl Process {
    pub fn new(id: ProcessId) -> Process {
        Process {
            id: id,
            state: State::Suspended,
            context: arch::Context::new(),
            priority: Priority::from(0),
        }
    }

    /// Set the state of the process
    pub fn set_state(&mut self, new: State) {
        self.state = new;
    }

    /// Set the stack pointer
    pub fn set_stack(&mut self, addr: usize) {
        self.context.set_stack(addr);
    }
}

/// A finishing process returns to this function
#[naked]
pub unsafe extern "C" fn process_return() {
    // For now, just print that a process returned
    println!("A process returned!");
}