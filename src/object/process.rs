use object::{HandleTable, UserHandle, HandleRights, Wasm, Thread};
use wasm::{Instance, VmCtx};
use cretonne_codegen::ir::TrapCode;
use nabi::Result;
use nil::{Ref, HandleRef};
use nil::mem::Bin;
use spin::RwLock;
use hashmap_core::HashMap;
use core::{mem, slice};
use sync::mpsc::Mpsc;
use arch::lock::Spinlock;
use alloc::Vec;

type ThreadList = Vec<Ref<Thread>>;

/// Represents a process.
#[derive(HandleRef)]
#[allow(dead_code)]
pub struct Process {
    /// The process name
    name: RwLock<Option<Bin<str>>>,
    /// Compiled code can be shared between processes.
    code: Ref<Wasm>,
    /// Process specific handle table.
    handle_table: RwLock<HandleTable>,
    /// List of threads operating in this
    /// process.
    thread_list: RwLock<ThreadList>,
    /// Hashmap of offsets in the wasm memory to an event
    pfex_map: Spinlock<HashMap<u32, Mpsc<*const Thread>>>,
    initial_instance: Instance,
}

impl Process {
    /// Create a process from already existing code.
    /// This is the only way to create a process.
    pub fn create(code: Ref<Wasm>) -> Result<Ref<Process>> {
        let initial_instance = code.generate_instance()?;

        Ref::new(Process {
            name: RwLock::new(None),
            code,
            handle_table: RwLock::new(HandleTable::new()),
            thread_list: RwLock::new(Vec::with_capacity(1)),
            pfex_map: Spinlock::new(HashMap::new()),
            initial_instance,
        })
    }

    pub fn create_thread(self: &Ref<Self>, func_addr: *const (), arg: u32, stack_ptr: u32) -> Result<UserHandle<Thread>> {
        let process = self.clone();

        let entry_point: extern fn(u32, &VmCtx) = unsafe { mem::transmute(func_addr) };

        let mut instance = self.initial_instance.clone();

        // assume that the first global is the simulated stack pointer
        let globals = unsafe { slice::from_raw_parts_mut(instance.globals.as_mut_ptr() as *mut usize, instance.globals.len() / mem::size_of::<usize>()) };
        globals[0] = stack_ptr as usize;

        let thread = Thread::new_with_parent(self.clone(), 1024 * 1024, move || {
            let mut vmctx_gen = instance.generate_vmctx_backing();
            
            let vmctx = vmctx_gen.vmctx(process, instance);
            entry_point(arg, vmctx);
        })?;

        self.thread_list.write().push(thread.clone());

        thread.resume();

        let mut handle_table = self.handle_table.write();

        handle_table.allocate(thread, HandleRights::WRITE | HandleRights::READ)
    }

    /// Start the process by spawning a thread at the entry point.
    pub fn start(self: &Ref<Self>) -> Result<()> {
        let process = self.clone();

        let mut instance = self.initial_instance.clone();

        let thread = Thread::new_with_parent(self.clone(), 1024 * 1024, move || {
            let entry_point = process.code.start_func();

            let mut vmctx_gen = instance.generate_vmctx_backing();
            let vmctx = vmctx_gen.vmctx(process, instance);
            entry_point(vmctx);
        })?;

        self.thread_list.write().push(thread.clone());

        thread.resume();
        
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

        let current_thread = Thread::current();

        // here, we need to kill all the threads in the process
        // except the current thread.
        let mut thread_list = self
            .thread_list
            .write();
        
        thread_list
            .drain(..)
            .filter(|thread| current_thread as *const Thread != &**thread as *const Thread)
            .for_each(|thread| {
                thread.exit().expect("unable to kill thread");
            });

        let current_thread = unsafe { Ref::from_raw(current_thread) };
        current_thread.exit().unwrap();
    }

    pub fn name(&self) -> &RwLock<Option<Bin<str>>> {
        &self.name
    }

    pub fn handle_table(&self) -> &RwLock<HandleTable> {
        &self.handle_table
    }

    pub fn thread_list(&self) -> &RwLock<ThreadList> {
        &self.thread_list
    }

    pub fn code(&self) -> &Wasm {
        &*self.code
    }

    pub fn pfex_map(&self) -> &Spinlock<HashMap<u32, Mpsc<*const Thread>>> {
        &self.pfex_map
    }

    pub fn initial_instance(&self) -> &Instance {
        &self.initial_instance
    }
}
