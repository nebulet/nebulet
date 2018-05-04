use common::table::{Table, TableIndex};
use object::handle::HandleTable;
use memory::Code;
use super::thread::Thread;
use arch::lock::Spinlock;

use alloc::arc::Arc;

use nabi::Result;

use wasm::compile_module;

pub struct Process {
    code: Code,
    handle_table: HandleTable,
    thread_table: Table<Arc<Spinlock<Thread>>>,
    started: bool
}

impl Process {
    /// Create a process from wasm.
    pub fn create(wasm_bytes: &[u8]) -> Result<Self> {
        let code = compile_module(wasm_bytes)?;

        Ok(Process {
            code: code,
            handle_table: HandleTable::new(),
            thread_table: Table::new(),
            started: false,
        })
    }

    /// Start the process by spawning a thread at the entry point.
    pub fn start(&mut self) -> Result<()> {
        self.started = true;

        self.code.execute();
        Ok(())
    }
}