use object::handle::{HandleTable, HandleRights};
use common::table::TableIndex;
use memory::Code;
use super::thread::Thread;

use alloc::Vec;
use alloc::boxed::Box;
use alloc::arc::Arc;

use nabi::Result;

use wasm::compile_module;
use wasm::runtime::Instance;

#[allow(dead_code)]
pub struct Process {
    code: Arc<Code>,
    handle_table: HandleTable,
    threads: Vec<TableIndex>,
    instance: Instance,
    started: bool
}

impl Process {
    /// Create a process from wasm.
    pub fn compile(wasm_bytes: &[u8]) -> Result<Self> {
        let code = compile_module(wasm_bytes)?;

        Self::create(Arc::new(code))
    }

    /// Create a process with already existing code.
    pub fn create(code: Arc<Code>) -> Result<Self> {
        let instance = code.generate_instance();

        Ok(Process {
            code,
            handle_table: HandleTable::new(),
            // since wasm only supports one thread rn...
            threads: Vec::with_capacity(1),
            instance,
            started: false,
        })
    }

    /// Start the process by spawning a thread at the entry point.
    /// The handle of `0` will always be the initial thread.
    pub fn start(&mut self) -> Result<()> {
        self.started = true;

        let thread = Thread::new(1024 * 16, common_process_entry, self as *mut Process as usize)?;

        let thread_handle = self.handle_table.allocate(thread, HandleRights::DUPLICATE | HandleRights::MUTABLE)?;
        self.threads.push(thread_handle);

        thread.resume()?;
        
        Ok(())
    }

    pub fn code(&self) -> Arc<Code> {
        self.code.clone()
    }
}

extern fn common_process_entry(arg: usize) {
    let process = unsafe {
        &mut *(arg as *mut Process)
    };

    let mut vmctx_backing = process.instance.generate_vmctx_backing();
    let vmctx = Box::new(vmctx_backing.vmctx(process));

    process.code.execute(&vmctx);
}
