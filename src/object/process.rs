use object::{HandleTable, CodeRef, Thread};
use wasm::Instance;
use cretonne_codegen::ir::TrapCode;
use nabi::Result;
use nil::{Ref, KernelRef};
use nil::mem::{Bin, Array};
use spin::RwLock;
use arch::cpu::Local;

type ThreadList = Array<Ref<Thread>>;

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
    pub fn start(self: &Ref<Self>) -> Result<()> {
        let process = self.clone();

        let thread = Thread::new(self.clone(), 1024 * 1024, move || {
            let entry_point = process.code.start_func();
            let mut vmctx_gen = process.instance.write().generate_vmctx_backing();
            let vmctx_ref = vmctx_gen.vmctx(process);
            entry_point(vmctx_ref);
        })?;

        self.thread_list.write().push(thread.clone())?;

        thread.resume()?;
        
        Ok(())
    }

    /// You just activated my trap card!
    /// 
    /// Being serious, almost all types of traps
    /// entail a process shutdown. Cretonne does
    /// support resumable traps, but they're not
    /// currently used.
    pub fn handle_trap(&self, trap_code: TrapCode) {
        println!("Trap: \"{}\"", trap_code);

        let current_thread = Local::current_thread();

        // here, we need to kill all the threads in the process
        // except the current thread. Since wasm currently
        // only supports a single thread, this will always do nothing
        // for now.
        self.thread_list
            .read()
            .iter()
            .filter(|thread| !thread.ptr_eq(&current_thread))
            .for_each(|thread| {
                thread.exit().expect("unable to kill thread");
            });

        current_thread.exit().unwrap();
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
