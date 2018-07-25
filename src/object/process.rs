use object::{HandleTable, Wasm, Thread};
use wasm::{Instance, VmCtx};
use cranelift_codegen::ir::TrapCode;
use nabi::Result;
use nil::mem::Bin;
use spin::RwLock;
use common::table::Table;
use hashmap_core::HashMap;
use core::{mem, slice};
use sync::mpsc::IntrusiveMpsc;
use arch::lock::Spinlock;
use alloc::boxed::Box;
use super::dispatcher::{Dispatch, Dispatcher};

/// Represents a process.
#[allow(dead_code)]
pub struct Process {
    /// The process name
    name: RwLock<Option<Bin<str>>>,
    /// Compiled code can be shared between processes.
    code: Dispatch<Wasm>,
    /// Process specific handle table.
    handle_table: RwLock<HandleTable>,
    /// List of threads operating in this
    /// process.
    thread_list: RwLock<Table<Box<Thread>>>,
    /// Hashmap of offsets in the wasm memory to an event
    pfex_map: Spinlock<HashMap<u32, IntrusiveMpsc<Thread>>>,
    initial_instance: Instance,
}

impl Process {
    /// Create a process from already existing code.
    /// This is the only way to create a process.
    pub fn create(code: Dispatch<Wasm>) -> Result<Dispatch<Self>> {
        let initial_instance = code.generate_instance()?;

        Ok(Dispatch::new(Process {
            name: RwLock::new(None),
            code,
            handle_table: RwLock::new(HandleTable::new()),
            thread_list: RwLock::new(Table::new()),
            pfex_map: Spinlock::new(HashMap::new()),
            initial_instance,
        }))
    }

    pub fn create_thread(self: &Dispatch<Self>, func_addr: *const (), arg: u32, stack_ptr: u32) -> Result<u32> {
        let process = self.copy_ref();

        let entry_point: extern fn(u32, &VmCtx) = unsafe { mem::transmute(func_addr) };

        let mut instance = self.initial_instance.clone();

        // assume that the first global is the simulated stack pointer
        let globals = unsafe { slice::from_raw_parts_mut(instance.globals.as_mut_ptr() as *mut usize, instance.globals.len() / mem::size_of::<usize>()) };
        globals[0] = stack_ptr as usize;

        let mut thread_list = self.thread_list.write();

        let id = thread_list.next_slot();

        let mut thread = Thread::new_with_parent(self.copy_ref(), id, 1024 * 1024, move || {
            let mut vmctx_gen = instance.generate_vmctx_backing();
            
            let vmctx = vmctx_gen.vmctx(process, instance);
            entry_point(arg, vmctx);
        })?;

        thread.start();

        let thread_id = thread_list.allocate(thread);

        debug_assert!(thread_id == id);
        debug_assert!(thread_id.inner() <= u32::max_value() as usize);

        Ok(thread_id.inner() as u32)
    }

    /// Start the process by spawning a thread at the entry point.
    pub fn start(self: &Dispatch<Self>) -> Result<()> {
        let process = self.copy_ref();

        let mut instance = self.initial_instance.clone();

        let mut thread_list = self.thread_list().write();

        let thread_id = thread_list.next_slot();

        debug_assert!(thread_id.inner() == 0);

        let mut thread = Thread::new_with_parent(self.copy_ref(), thread_id, 1024 * 1024, move || {
            let entry_point = process.code.start_func();

            let mut vmctx_gen = instance.generate_vmctx_backing();
            let vmctx = vmctx_gen.vmctx(process, instance);
            entry_point(vmctx);
        })?;

        thread.start();

        let id = thread_list.allocate(thread);

        debug_assert!(id == thread_id);
        debug_assert!(id.inner() == 0);
        
        Ok(())
    }

    pub fn exit(&self) {
        let current_thread = Thread::current();

        // here, we need to kill all the threads in the process
        // except the current thread.
        {
            let mut thread_list = self
                .thread_list
                .write();
            
            thread_list
                .drain(..)
                .filter(|thread| current_thread as *const Thread != &**thread as *const Thread)
                .for_each(|thread| {
                    thread.kill();
                });

            assert!(thread_list.len() == 1);
        }

        Thread::exit();
    }

    /// You just activated my trap card!
    /// 
    /// Being serious, almost all types of traps
    /// entail a process shutdown. Cranelift does
    /// support resumable traps, but they're not
    /// currently used.
    pub fn handle_trap(&self, trap_code: TrapCode) {
        println!("Trap: \"{}\"", trap_code);

        self.exit();
    }

    pub fn name(&self) -> &RwLock<Option<Bin<str>>> {
        &self.name
    }

    pub fn handle_table(&self) -> &RwLock<HandleTable> {
        &self.handle_table
    }

    pub fn thread_list(&self) -> &RwLock<Table<Box<Thread>>> {
        &self.thread_list
    }

    pub fn code(&self) -> &Wasm {
        &*self.code
    }

    pub fn pfex_map(&self) -> &Spinlock<HashMap<u32, IntrusiveMpsc<Thread>>> {
        &self.pfex_map
    }

    pub fn initial_instance(&self) -> &Instance {
        &self.initial_instance
    }
}

impl Dispatcher for Process {}

impl Drop for Process {
    fn drop(&mut self) {
        println!("process dropping.");
    }
}
