//! A `Compilation` contains the compiled function bodies for a WebAssembly
//! module

use super::module::Module;
use super::instance::Instance;
use super::{Relocations, Relocation, DataInitializer};
use cretonne::{self, result::CtonError, isa::TargetIsa};
use super::RelocSink;

use memory::{Code, Region, sip};

use nabi::{Result, Error};

use core::slice;
use core::ptr::{write_unaligned, NonNull};
use alloc::Vec;

#[derive(Debug)]
pub struct Compilation<'module> {
    /// The module this is instantiated from
    module: &'module Module,

    instance: Instance,

    region: Region,

    /// Compiled machine code for the function bodies
    /// This is mapped onto `self.region`.
    functions: Vec<&'module [u8]>,

    /// The computed relocations
    relocations: Relocations,
}

impl<'module> Compilation<'module> {
    /// Allocates the runtime data structures with the given flags
    pub fn new(module: &'module Module, region: Region, functions: Vec<&'module [u8]>, relocations: Relocations, instance: Instance) -> Self {
        Compilation {
            module,
            region,
            instance,
            functions,
            relocations,
        }
    }

    /// Relocate the compliation.
    fn relocate(&mut self) {
        // The relocations are relative to the relocation's address plus four bytes
        // TODO: Support architectures other than x86_64, and other reloc kinds.
        for (i, function_relocs) in self.relocations.iter().enumerate() {
            for ref r in function_relocs {
                let target_func_addr: isize = self.functions[r.func_index].as_ptr() as isize;
                let body = self.functions[i];
                unsafe {
                    let reloc_addr = body.as_ptr().offset(r.offset as isize) as isize;
                    let reloc_addend = r.addend as isize - 4;
                    let reloc_delta_i32 = (target_func_addr - reloc_addr + reloc_addend) as i32;
                    write_unaligned(reloc_addr as *mut i32, reloc_delta_i32);
                }
            }
        }
    }

    /// Emit a `Code` instance
    pub fn emit(mut self) -> Code<'module> {
        self.relocate();

        let vmctx = self.instance.generate_vmctx();

        let start_index = self.module.start_func
            .expect("No start function");
        
        let start_ptr = self.functions[start_index].as_ptr();

        Code::new(self.module, self.region, self.instance, vmctx, start_ptr)
    }
}

/// Define functions, etc and then "compile"
/// it all into a `Compliation`.
pub struct Compiler<'module, 'isa> {
    /// The module this is instantiated from
    module: &'module Module,

    isa: &'isa TargetIsa,

    contexts: Vec<(cretonne::Context, usize)>,

    total_size: usize,
}

impl<'module, 'isa> Compiler<'module, 'isa> {
    pub fn new(module: &'module Module, isa: &'isa TargetIsa) -> Self {
        Self::with_capacity(module, isa, 0)
    }

    pub fn with_capacity(module: &'module Module, isa: &'isa TargetIsa, capacity: usize) -> Self {
        Compiler {
            module,
            isa,
            contexts: Vec::with_capacity(capacity),
            total_size: 0,
        }
    }

    /// Define a function. This also compiles the function.
    pub fn define_function(&mut self, mut ctx: cretonne::Context) -> Result<()> {
        let code_size = ctx.compile(self.isa)
            .map_err(|_| Error::INTERNAL)? as usize;

        self.contexts.push((ctx, code_size));

        self.total_size += code_size;

        Ok(())
    }
    
    /// This allocates a region from the Sip memory allocator
    /// and emits all the functions into that.
    /// 
    /// This assumes that the functions don't need a specific
    /// alignment, which is true on x86_64, but may not
    /// be true on other architectures.
    pub fn compile(self, data_initializers: &[DataInitializer]) -> Result<Compilation<'module>> {
        let mut region = sip::allocate_region(self.total_size)
            .ok_or(Error::NO_MEMORY)?;

        let mut offset = region.start().as_u64() as usize;
        let mut functions = Vec::with_capacity(self.contexts.len());
        let mut relocs = Vec::with_capacity(self.contexts.len());

        // emit functions to memory
        for (ref ctx, size) in self.contexts.iter() {
            let mut reloc_sink = RelocSink::new(&ctx.func);
            ctx.emit_to_memory(offset as *mut u8, &mut reloc_sink, self.isa);
            functions.push(unsafe {
                slice::from_raw_parts(offset as *const u8, *size).into()
            });
            relocs.push(reloc_sink.func_relocs);

            offset += size;
        }

        let instance = Instance::new(self.module, data_initializers);

        Ok(Compilation::new(self.module, region, functions, relocs, instance))
    }
}