//! An 'Instance' contains all the runtime state used by execution of a wasm module
//!
//! Literally taken from https://github.com/sunfishcode/wasmstandalone

use cretonne_codegen::ir;
use cretonne_wasm::{GlobalIndex, GlobalInit};
use super::module::Module;
use super::{DataInitializer, FunctionIndex};

use memory::WasmMemory;
use object::ProcessRef;
use nil::Ref;
use core::marker::PhantomData;
use core::{slice, mem};
use alloc::Vec;
use common::slice::{BoundedSlice, UncheckedSlice};

pub fn get_function_addr(base: *const (), functions: &[usize], func_index: FunctionIndex) -> *const () {
    let offset = functions[func_index];
    (base as usize + offset) as _
}

pub struct VmCtxBacking {
    globals: UncheckedSlice<u8>,
    memories: Vec<UncheckedSlice<u8>>,
    tables: Vec<BoundedSlice<usize>>,
}

impl VmCtxBacking {
    pub fn vmctx(&mut self, process: Ref<ProcessRef>) -> VmCtx {
        VmCtx {
            globals: self.globals,
            memories: (&*self.memories).into(),
            tables: (&*self.tables).into(),
            process,
            phantom: PhantomData,
        }
    }
}

#[repr(C)]
pub struct VmCtx<'a> {
    globals: UncheckedSlice<u8>,
    memories: UncheckedSlice<UncheckedSlice<u8>>,
    tables: UncheckedSlice<BoundedSlice<usize>>,
    pub process: Ref<ProcessRef>,
    phantom: PhantomData<&'a ()>,
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
    pub fn new(module: &Module, data_initializers: &[DataInitializer], code_base: *const (), functions: &[usize]) -> Instance {
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
            .map(|mem| {
                let slice: &[u8] = &*mem;
                slice.into()
            })
            .collect();

        let tables = self.tables.iter_mut()
            .map(|table| {
                let slice: &[usize] = &*table;
                slice.into()
            })
            .collect();
        
        VmCtxBacking {
            memories,
            tables,
            globals: (&*self.globals).into(),
        }
    }

    /// Allocate memory in `self` for just the tables of the current module.
    fn instantiate_tables(&mut self, module: &Module, code_base: *const (), functions: &[usize]) {
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
            debug_assert!(table_element.base.is_none(), "globalvar base not supported yet.");
            let base = 0;

            let table = &mut self.tables[table_element.table_index];
            for (i, elem) in table_element.elements.iter().enumerate() {
                // since the table just contains functions in the MVP
                // we get the address of the specified function indexes
                // to populate the table.
                let func_index = *elem - module.imported_funcs.len();
                let func_addr = get_function_addr(code_base, functions, func_index);
                table[base + table_element.offset + i] = func_addr as _;
            }
        }
    }

    /// Allocate memory in `self` for just the memories of the current module.
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

        // the wasm memories are lazily mapped,
        // so we need to be careful to map
        // in the pages that get initialized here.
        for init in data_initializers {
            debug_assert!(init.base.is_none(), "globalvar base not supported yet.");
            let memory = &mut self.memories[init.memory_index];

            let start_offset = init.offset;
            let end_offset = init.offset + init.data.len();

            memory.map_range(start_offset, end_offset).unwrap();

            let to_init = &mut memory[start_offset..end_offset];
            to_init.copy_from_slice(&init.data);
        }
    }

    /// Allocate memory in `self` for just the globals of the current module.
    fn instantiate_globals(&mut self, module: &Module) {
        debug_assert!(self.globals.is_empty());
        let globals_count = module.globals.len();
        // Allocate the underlying memory and initialize it to zeros
        let globals_data_size = globals_count * 8;
        self.globals.resize(globals_data_size, 0);

        // cast the self.globals slice to a slice of i64.
        let globals_data = unsafe { slice::from_raw_parts_mut(self.globals.as_mut_ptr() as *mut i64, globals_count) };
        for (i, global) in module.globals.iter().enumerate() {
            let value: i64 = match global.initializer {
                GlobalInit::I32Const(n) => n as _,
                GlobalInit::I64Const(n) => n,
                GlobalInit::F32Const(f) => unsafe { mem::transmute(f as f64) },
                GlobalInit::F64Const(f) => unsafe { mem::transmute(f) },
                _ => unimplemented!(),
            };
            
            globals_data[i] = value;
        }
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
