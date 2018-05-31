use wasm::instance::{Instance, VmCtx};
use wasm::{Module, ModuleEnvironment, DataInitializer};
use memory::{Region, MemFlags};
use nabi::Result;
use core::mem;
use alloc::Vec;
use nil::{Ref, KernelRef};
use cretonne_codegen::settings::{self, Configurable};
use cretonne_wasm::translate_module;
use cretonne_native;

/// A `CodeRef` represents
/// webassembly code compiled
/// into machine code. You must
/// have one of these to create
/// a process.
#[allow(dead_code)]
#[derive(KernelRef)]
pub struct CodeRef {
    data_initializers: Vec<DataInitializer>,
    module: Module,
    region: Region,
    start_func: extern fn(&VmCtx),
}

impl CodeRef {
    /// Compile webassembly bytecode into a CodeRef.
    pub fn compile(wasm: &[u8]) -> Result<Ref<CodeRef>> {
        let (mut flag_builder, isa_builder) = cretonne_native::builders()
        .map_err(|_| internal_error!())?;

        flag_builder.set("opt_level", "best")
            .map_err(|_| internal_error!())?;

        let isa = isa_builder.finish(settings::Flags::new(flag_builder));

        let module = Module::new();
        let mut environ = ModuleEnvironment::new(isa.flags(), module);

        translate_module(wasm, &mut environ)
            .map_err(|_| internal_error!())?;

        let translation = environ.finish_translation();
        let (compliation, module, data_initializers) = translation.compile(&*isa)?;

        compliation.emit(module, data_initializers)
    }

    /// Used for internal use.
    pub fn new(module: Module, data_initializers: Vec<DataInitializer>, mut region: Region, start_func: *const ())
        -> Result<Ref<CodeRef>>
    {
        let flags = MemFlags::READ | MemFlags::EXEC;
        region.remap(flags)?;

        assert!(region.contains(start_func as usize));

        let start_func = unsafe {
            mem::transmute(start_func)
        };

        Ref::new(CodeRef {
            data_initializers,
            module,
            region,
            start_func,
        })
    }

    pub fn generate_instance(&self) -> Instance {
        Instance::new(&self.module, &self.data_initializers)
    }

    pub fn start_func(&self) -> extern fn(&VmCtx) {
        self.start_func
    }
}
