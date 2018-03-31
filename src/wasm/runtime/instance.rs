//! An 'Instance' contains all the runtime state used by execution of a wasm module
//! 
//! Literally taken from https://github.com/sunfishcode/wasmstandalone

use cretonne::ir;
use cton_wasm::GlobalIndex;
use super::module::Module;
use super::DataInitializer;

use memory::{WasmMemory, sip};

use alloc::Vec;

#[repr(C)]
#[derive(Debug)]
pub struct VmCtx {
    /// Single global buffer.
    globals: *mut u8,
    /// Array of memories.
    /// (Only supports 1 for now.)
    memories: *const *mut u8,
}

/// An Instance of a WebAssembly module
#[derive(Debug)]
pub struct Instance {
    /// WebAssembly table data
    pub tables: Vec<Vec<usize>>,

    /// WebAssembly linear memory data
    pub memories: Vec<WasmMemory>,

    /// WebAssembly global variable data
    pub globals: Vec<u8>,

    memory_array_backing: Vec<*mut u8>, 
}

impl Instance {
    /// Create a new `Instance`.
    pub fn new(module: &Module, data_initializers: &[DataInitializer]) -> Instance {
        let mut result = Instance {
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            memory_array_backing: Vec::new(),
        };

        result.instantiate_tables(module);
        result.instantiate_memories(module, data_initializers);
        result.instantiate_globals(module);
        result
    }

    /// Create the VmCtx data structure for the JIT'd code to use. This must
    /// match the VmCtx layout in the runtime.
    // pub fn generate_vmctx(&mut self) -> Vec<*mut u8> {
    //     let mut memories: Vec<*mut u8> = Vec::with_capacity(self.memories.len());
    //     let mut vmctx = Vec::with_capacity(2);
    //     vmctx.push(self.globals.as_mut_ptr());
    //     for mem in &mut self.memories {
    //         println!("memory slice: [{:#x}; {}]", mem.as_ptr() as usize, mem.len());
    //         memories.push(mem.as_mut_ptr());
    //     }
    //     println!("memories.as_ptr(): {:#x}", memories.as_ptr() as usize);
    //     vmctx.push(memories.as_mut_ptr() as *mut u8);
    //     println!("vmctx.as_ptr(): {:#x}", vmctx.as_ptr() as usize);
    //     vmctx
    // }

    pub fn generate_vmctx(&mut self) -> VmCtx {
        let vmctx = VmCtx {
            globals: self.globals.as_mut_ptr(),
            memories: self.memory_array_backing.as_mut_ptr(),
        };
        vmctx
    }

    /// Allocate memory in `self` for just the tables of the current module,
    /// without initializers applied just yet.
    fn instantiate_tables(&mut self, module: &Module) {
        debug_assert!(self.tables.is_empty());

        self.tables.reserve_exact(module.tables.len());
        for table in &module.tables {
            let len = table.size;
            let mut v = Vec::with_capacity(len);
            v.resize(len, 0);
            self.tables.push(v);
        }
    }

    /// Allocate memory in `self` for just the memories of the current module,
    /// without any initializers applied just yet
    fn instantiate_memories(&mut self, module: &Module, data_initializers: &[DataInitializer]) {
        debug_assert!(self.memories.is_empty());
        // Allocate the underlying memory and initialize it to all zeros
        self.memories.reserve_exact(module.memories.len());
        self.memory_array_backing.reserve_exact(self.memories.len());

        for memory in &module.memories {
            let mut heap = sip::allocate_wasm_memory()
                .expect("Could not allocate wasm memory");
            heap.grow(memory.pages_count)
                .expect("Could not grow wasm heap to initial size");

            self.memory_array_backing.push(heap.as_mut_ptr());
            self.memories.push(heap);
        }
        for init in data_initializers {
            debug_assert!(init.base.is_none(), "globalvar base not supported yet.");

            let to_init = &mut self.memories[init.memory_index][init.offset..init.offset + init.data.len()];
            to_init.copy_from_slice(init.data);
        }
    }

    /// Allocate memory in `self` for just the globals of the current module,
    /// without any initializers applied just yet.
    fn instantiate_globals(&mut self, module: &Module) {
        debug_assert!(self.globals.is_empty());
        // Allocate the underlying memory and initialize it to zeros
        let globals_data_size = module.globals.len() * 8;
        self.globals.resize(globals_data_size, 0);
    }

    /// Returns a slice of the contents of allocated linear memory
    pub fn inspect_memory(&self, memory_index: usize, address: usize, len: usize) -> &[u8] {
        &self.memories.get(memory_index).expect(
            format!("no memory for index {}", memory_index).as_str()
        )[address..address + len]
    }

    /// Return the value of a global variable.
    pub fn inspect_globals(&self, global_index: GlobalIndex, ty: ir::Type) -> &[u8] {
        let offset = global_index * 8;
        let len = ty.bytes() as usize;
        &self.globals[offset..offset + len]
    }
}