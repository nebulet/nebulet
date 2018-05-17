use object::handle::{HandleTable, HandleRights};
use common::table::TableIndex;
use memory::Code;
use wasm::runtime::Instance;
use wasm::compile_module;
use task::Thread;

use nabi::Result;

use alloc::arc::Arc;
use alloc::{String, Vec};
use alloc::boxed::Box;

pub struct Process {
    /// The process name
    pub name: String,
    /// Compiled code can be shared between processes.
    code: Arc<Code>,
    /// Process specific handle table.
    pub handle_table: HandleTable,
    /// List of threads (referring to their index in the handle table).
    pub threads: Vec<TableIndex>,
    /// A process owns its own instance.
    pub instance: Instance,
    pub started: bool
}

impl Process {
    /// Create a process from wasm.
    pub fn compile<S: Into<String>>(name: S, wasm_bytes: &[u8]) -> Result<Process> {
        let code = compile_module(wasm_bytes)?;

        Self::create(name, Arc::new(code))
    }

    /// Create a process with already existing code.
    pub fn create<S: Into<String>>(name: S, code: Arc<Code>) -> Result<Process> {
        let instance = code.generate_instance();

        Ok(Process {
            name: name.into(),
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
    pub fn start(&mut self, global_proc_index: TableIndex) -> Result<()> {
        self.started = true;

        let thread = Thread::new(1024 * 16, common_process_entry, global_proc_index)?;

        let thread_handle = self.handle_table.allocate(thread, HandleRights::DUPLICATE | HandleRights::MUTABLE)?;
        self.threads.push(thread_handle);

        thread.resume()?;
        
        Ok(())
    }
}

/// This is the entry point for all processes.
extern fn common_process_entry(proc_index: usize) {
    use object::GlobalHandleTable;
    let (entry_point, vmctx) = {
        let handle_table = GlobalHandleTable::get();
        {
            let mut process = handle_table
                .get_handle(proc_index)
                .unwrap()
                .lock_cast::<Process>()
                .unwrap();
        
            let mut vmctx_backing = process.instance.generate_vmctx_backing();
            let vmctx = Box::new(vmctx_backing.vmctx(proc_index));

            (process.code.start_func(), vmctx)
        }
    };

    entry_point(&vmctx);
}
