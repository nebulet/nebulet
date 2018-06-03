//! An 'Instance' contains all the runtime state used by execution of a wasm module
//!
//! Literally taken from https://github.com/sunfishcode/wasmstandalone

use cretonne_codegen::ir;
use cretonne_wasm::GlobalIndex;
use super::module::Module;
use super::{DataInitializer, FunctionIndex};
use super::compilation::{FunctionType, get_abi_func};

use memory::WasmMemory;
use object::ProcessRef;
use nil::Ref;
use nabi::Result;

use core::ptr::NonNull;
use core::marker::PhantomData;
use alloc::Vec;

pub fn get_function_addr(base: *const (), functions: &[FunctionType], module_ref: &Module, func_index: FunctionIndex) -> Result<*const ()> {
        match functions[func_index] {
            FunctionType::Local {
                offset,
                size: _,
            } => {
                Ok((base as usize + offset) as _)
            },
            FunctionType::External {
                ref module,
                ref name,
            } => {
                match module.as_str() {
                    "abi" => {
                        let sig_index = module_ref.functions[func_index];
                        let imported_sig = &module_ref.signatures[sig_index];

                        get_abi_func(name, imported_sig)
                    },
                    _ => {
                        Err(internal_error!())
                    }
                }
            },
        }
    }

pub struct VmCtxBacking {
    globals: NonNull<u8>,
    memories: Vec<NonNull<u8>>,
    tables: Vec<NonNull<usize>>,
}

impl VmCtxBacking {
    pub fn vmctx(&mut self, process: Ref<ProcessRef>) -> VmCtx {
        VmCtx {
            globals: self.globals,
            memories: NonNull::new(self.memories.as_mut_ptr()).unwrap(),
            tables: NonNull::new(self.tables.as_mut_ptr()).unwrap(),
            process,
            _phantom: PhantomData,
        }
    }
}

#[repr(C)]
pub struct VmCtx<'a> {
    globals: NonNull<u8>,
    memories: NonNull<NonNull<u8>>,
    tables: NonNull<NonNull<usize>>,
    pub process: Ref<ProcessRef>,
    _phantom: PhantomData<&'a ()>,
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
}

impl Instance {
    /// Create a new `Instance`.
    pub fn new(module: &Module, data_initializers: &[DataInitializer], code_base: *const (), functions: &[FunctionType]) -> Instance {
        let mut result = Instance {
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
        };

        result.instantiate_tables(module, code_base, functions);
        result.instantiate_memories(module, data_initializers);
        result.instantiate_globals(module);
        result
    }

    pub fn generate_vmctx_backing(&mut self) -> VmCtxBacking {
        let memories = self.memories.iter_mut()
            .map(|mem| NonNull::new(mem.as_mut_ptr()).unwrap())
            .collect();

        let tables = self.tables.iter_mut()
            .map(|table| NonNull::new(table.as_mut_ptr()).unwrap())
            .collect();
        
        VmCtxBacking {
            memories,
            tables,
            globals: NonNull::new(self.globals.as_mut_ptr()).unwrap(),
        }
    }

    /// Allocate memory in `self` for just the tables of the current module,
    /// without initializers applied just yet.
    fn instantiate_tables(&mut self, module: &Module, code_base: *const (), functions: &[FunctionType]) {
        debug_assert!(self.tables.is_empty());

        self.tables.reserve_exact(module.tables.len());
        for table in &module.tables {
            let len = table.size;
            let mut v = Vec::with_capacity(len);
            v.resize(len, 0);
            self.tables.push(v);
        }
        // instantiate tables
        for table_element in &module.table_elements {
            // let base = table_element.base.map_or(0, |base| {
                
            // });
            let base = 0;

            let table = &mut self.tables[table_element.table_index];
            for (i, elem) in table_element.elements.iter().enumerate() {
                // since the table just contains functions in the MVP
                // we get the address of the specified function indexes
                // populate the table.
                let func_addr = get_function_addr(code_base, functions, module, *elem).unwrap();
                table[base + table_element.offset + i] = func_addr as _;
            }
        }
    }

    /// Allocate memory in `self` for just the memories of the current module,
    /// without any initializers applied just yet
    fn instantiate_memories(&mut self, module: &Module, data_initializers: &[DataInitializer]) {
        debug_assert!(self.memories.is_empty());
        // Allocate the underlying memory and initialize it to all zeros
        self.memories.reserve_exact(module.memories.len());

        for memory in &module.memories {
            let mut heap = WasmMemory::allocate()
                .expect("Could not allocate wasm memory");
            heap.grow(memory.pages_count)
                .expect("Could not grow wasm heap to initial size");
            self.memories.push(heap);
        }
        for init in data_initializers {
            debug_assert!(init.base.is_none(), "globalvar base not supported yet.");

            let to_init = &mut self.memories[init.memory_index][init.offset..init.offset + init.data.len()];
            to_init.copy_from_slice(&init.data);
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
