//! An 'Instance' contains all the runtime state used by execution of a wasm module
//!
//! Literally taken from https://github.com/sunfishcode/wasmstandalone

use cretonne_wasm::{GlobalInit};
use super::module::Module;
use super::{DataInitializer, FunctionIndex};

use memory::WasmMemory;
use object::{Dispatch, Process};
use nabi::Result;
use core::marker::PhantomData;
use core::{slice, mem};
use alloc::Vec;
use alloc::arc::Arc;
use spin::RwLock;
use common::slice::{BoundedSlice, UncheckedSlice};

pub fn get_function_addr(base: *const (), functions: &[usize], func_index: FunctionIndex) -> *const () {
    let offset = functions[func_index];
    (base as usize + offset) as _
}

pub struct VmCtxGenerator {
    globals: UncheckedSlice<u8>,
    memories: Vec<UncheckedSlice<u8>>,
    tables: Vec<BoundedSlice<usize>>,
}

impl VmCtxGenerator {
    pub fn vmctx(&mut self, process: Dispatch<Process>, instance: Instance) -> &VmCtx {
        assert!(self.memories.len() >= 1, "modules must have at least one memory");
        // the first memory has a space of `mem::size_of::<VmCtxData>()` rounded
        // up to the 4KiB before it. We write the VmCtxData into that.
        let data = VmCtxData {
            globals: self.globals,
            memories: self.memories[1..].into(),
            tables: self.tables[..].into(),
            user_data: UserData {
                process,
                instance,
            },
            phantom: PhantomData,
        };

        let main_heap_ptr = self.memories[0].as_mut_ptr() as *mut VmCtxData;
        unsafe {
            main_heap_ptr
                .sub(1)
                .write(data);
            &*(main_heap_ptr as *const VmCtx)
        }
    }
}

/// Zero-sized, non-instantiable type.
pub enum VmCtx {}

impl VmCtx {
    pub fn data(&self) -> &VmCtxData {
        let heap_ptr = self as *const _ as *const VmCtxData;
        unsafe {
            &*heap_ptr.sub(1)
        }
    }

    /// This is safe because the offset is 32 bits and thus
    /// cannot extend out of the guarded wasm memory.
    pub fn fastpath_offset_ptr<T>(&self, offset: u32) -> *const T {
        let heap_ptr = self as *const _ as *const u8;
        unsafe {
            heap_ptr.add(offset as usize) as *const T
        }
    }
}

#[repr(C)]
pub struct VmCtxData<'a> {
    pub user_data: UserData,
    globals: UncheckedSlice<u8>,
    memories: UncheckedSlice<UncheckedSlice<u8>>,
    tables: UncheckedSlice<BoundedSlice<usize>>,
    phantom: PhantomData<&'a ()>,
}

#[repr(C)]
pub struct UserData {
    pub process: Dispatch<Process>,
    pub instance: Instance,
}

struct InstanceBuilder {
    tables: Vec<Vec<usize>>,
    memories: Vec<WasmMemory>,
    globals: Vec<u8>,
}

impl InstanceBuilder {
    pub fn new(module: &Module, data_initializers: &[DataInitializer], code_base: *const (), functions: &[usize]) -> InstanceBuilder {
        let mut builder = InstanceBuilder {
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
        };

        builder.instantiate_tables(module, code_base, functions);
        builder.instantiate_memories(module, data_initializers);
        builder.instantiate_globals(module);

        builder
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
            assert!(table_element.base.is_none(), "globalvalue base not supported yet.");
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

        for (i, memory) in module.memories.iter().enumerate() {
            let pre_space = if i == 0 { // first memory
                mem::size_of::<VmCtxData>()
            } else {
                0
            };

            let mut heap = WasmMemory::allocate(pre_space)
                .expect("Could not allocate wasm memory");
            heap.grow(memory.pages_count)
                .expect("Could not grow wasm heap to initial size");
            self.memories.push(heap);
        }

        // the wasm memories are lazily mapped,
        // so we need to be caArcul to map
        // in the pages that get initialized here.
        for init in data_initializers {
            assert!(init.base.is_none(), "globalvalue base not supported yet.");
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
}

/// An Instance of a WebAssembly module
#[derive(Debug)]
pub struct Instance {
    /// WebAssembly table data
    pub tables: Arc<Vec<RwLock<Vec<usize>>>>,

    /// WebAssembly linear memory data
    pub memories: Arc<Vec<RwLock<WasmMemory>>>,

    /// WebAssembly global variable data
    pub globals: Vec<u8>,
}

impl Instance {
    /// Create a new `Instance`.
    pub fn build(module: &Module, data_initializers: &[DataInitializer], code_base: *const (), functions: &[usize]) -> Result<Instance> {
        let builder = InstanceBuilder::new(module, data_initializers, code_base, functions);

        Ok(Instance {
            tables: Arc::new(builder.tables.into_iter().map(|table| RwLock::new(table)).collect()),
            memories: Arc::new(builder.memories.into_iter().map(|mem| RwLock::new(mem)).collect()),
            globals: builder.globals,
        })
    }

    pub fn generate_vmctx_backing(&mut self) -> VmCtxGenerator {
        let memories = self.memories.iter()
            .map(|mem| mem.write()[..].into())
            .collect();

        let tables = self.tables.iter()
            .map(|table| table.write()[..].into())
            .collect();
        
        VmCtxGenerator {
            globals: self.globals[..].into(),
            memories,
            tables,
        }
    }

    pub fn memories(&self) -> Arc<Vec<RwLock<WasmMemory>>> {
        self.memories.clone()
    }
}

impl Clone for Instance {
    fn clone(&self) -> Instance {
        Instance {
            tables: Arc::clone(&self.tables),
            memories: Arc::clone(&self.memories),
            globals: self.globals.clone(),
        }
    }
}
