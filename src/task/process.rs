use object::handle::HandleTable;
use memory::Code;
use super::thread_entry::ThreadEntry;
use super::thread::Thread;

use alloc::Vec;

use nabi::Result;

use wasm::compile_module;

pub struct Process {
    code: Code,
    handle_table: HandleTable,
    threads: Vec<ThreadEntry>,
    started: bool
}

impl Process {
    /// Create a process from wasm.
    pub fn create(wasm_bytes: &[u8]) -> Result<Self> {
        let code = compile_module(wasm_bytes)?;

        Ok(Process {
            code: code,
            handle_table: HandleTable::new(),
            // since wasm only supports one thread rn...
            threads: Vec::with_capacity(1),
            started: false,
        })
    }

    /// Start the process by spawning a thread at the entry point.
    pub fn start(&mut self) -> Result<()> {
        self.started = true;
        
        Ok(())
    }
}