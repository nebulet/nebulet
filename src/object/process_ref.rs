use object::{HandleTable, HandleRights, CodeRef, ThreadRef};
use wasm::runtime::Instance;
use alloc::String;
use alloc::boxed::Box;
use nabi::Result;
use nil::{Ref, KernelRef};
use spin::RwLock;

#[derive(KernelRef)]
pub struct ProcessRef {
    /// The process name
    name: String,
    /// Compiled code can be shared between processes.
    code: Ref<CodeRef>,
    /// Process specific handle table.
    handle_table: RwLock<HandleTable>,
    /// A process owns its own instance.
    instance: RwLock<Instance>,
}

impl ProcessRef {
    /// Create a process with already existing code.
    pub fn create<S: Into<String>>(name: S, code: Ref<CodeRef>) -> Result<Ref<ProcessRef>> {
        let instance = code.generate_instance();

        Ok(ProcessRef {
            name: name.into(),
            code,
            handle_table: RwLock::new(HandleTable::new()),
            instance: RwLock::new(instance),
        }.into())
    }

    /// Start the process by spawning a thread at the entry point.
    /// The handle of `0` will always be the initial thread.
    pub fn start(self: Ref<Self>) -> Result<()> {
        let process = self.clone();

        let thread = ThreadRef::new(1024 * 1024, move || {
            let entry_point = process.code.start_func();
            let vmctx = {
                let mut vmctx_packing = process.instance.write().generate_vmctx_backing();
                Box::new(vmctx_packing.vmctx(process.clone()))
            };

            entry_point(&vmctx);
        })?;

        let _thread_handle = self.handle_table
            .write()
            .allocate(thread.clone(), HandleRights::READ | HandleRights::WRITE)?;

        thread.resume()?;
        
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
