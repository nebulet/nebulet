use object::{HandleTable, CodeRef, ThreadRef};
use wasm::Instance;
use nabi::Result;
use nil::{Ref, KernelRef};
use nil::mem::{Bin, Array};
use spin::RwLock;

type ThreadList = Array<Ref<ThreadRef>>;

/// Represents a process.
#[derive(KernelRef)]
#[allow(dead_code)]
pub struct ProcessRef {
    /// The process name
    name: RwLock<Option<Bin<str>>>,
    /// Compiled code can be shared between processes.
    code: Ref<CodeRef>,
    /// Process specific handle table.
    handle_table: RwLock<HandleTable>,
    /// List of threads operating in this
    /// process.
    thread_list: RwLock<ThreadList>,
    /// A process owns its own instance.
    instance: RwLock<Instance>,
}

impl ProcessRef {
    /// Create a process from already existing code.
    /// This is the only way to create a process.
    pub fn create(code: Ref<CodeRef>) -> Result<Ref<ProcessRef>> {
        let instance = code.generate_instance();

        Ref::new(ProcessRef {
            name: RwLock::new(None),
            code,
            handle_table: RwLock::new(HandleTable::new()),
            thread_list: RwLock::new(Array::with_capacity(1)?),
            instance: RwLock::new(instance),
        })
    }

    /// Start the process by spawning a thread at the entry point.
    /// The handle of `0` will always be the initial thread.
    pub fn start(self: &Ref<Self>) -> Result<()> {
        let process = self.clone();

        let thread = ThreadRef::new(1024 * 1024, move || {
            let entry_point = process.code.start_func();
            let mut vmctx_backing = process.instance.write().generate_vmctx_backing();
            let vmctx = vmctx_backing.vmctx(process);
            let vmctx_bin = Bin::new(vmctx).unwrap();
            entry_point(&vmctx_bin);
        })?;

        self.thread_list.write().push(thread.clone())?;

        thread.resume()?;
        
        Ok(())
    }

    pub fn name(&self) -> &RwLock<Option<Bin<str>>> {
        &self.name
    }

    pub fn handle_table(&self) -> &RwLock<HandleTable> {
        &self.handle_table
    }

    pub fn instance(&self) -> &RwLock<Instance> {
        &self.instance
    }

    pub fn thread_list(&self) -> &RwLock<ThreadList> {
        &self.thread_list
    }

    pub fn code(&self) -> &CodeRef {
        &*self.code
    }
}
