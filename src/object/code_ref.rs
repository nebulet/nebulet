use wasm::runtime::instance::{Instance, VmCtx};
use wasm::runtime::{Module, DataInitializer};
use wasm::compile_module;
use memory::{Region, MemFlags};
use nabi::Result;
use core::mem;
use alloc::Vec;
use nil::{Ref, KernelRef};

#[allow(dead_code)]
#[derive(KernelRef)]
pub struct CodeRef {
    data_initializers: Vec<DataInitializer>,
    module: Module,
    region: Region,
    start_func: extern fn(&VmCtx),
}

impl CodeRef {
    pub fn compile(wasm_bytes: &[u8]) -> Result<Ref<CodeRef>> {
        compile_module(wasm_bytes)
    }

    pub fn new(module: Module, data_initializers: Vec<DataInitializer>, mut region: Region, start_func: *const u8)
        -> Result<Ref<CodeRef>>
    {
        let flags = MemFlags::READ | MemFlags::EXEC;
        region.remap(flags)?;

        assert!(region.contains(start_func as usize));

        let start_func = unsafe {
            mem::transmute(start_func)
        };

        Ok(CodeRef {
            data_initializers,
            module,
            region,
            start_func,
        }.into())
    }

    pub fn generate_instance(&self) -> Instance {
        Instance::new(&self.module, &self.data_initializers)
    }

    pub fn start_func(&self) -> extern fn(&VmCtx) {
        self.start_func
    }
}
