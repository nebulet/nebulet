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
pub struct Compilation {
    /// The module this is instantiated from
    module: Module,

    instance: Instance,

    region: Region,

    /// Compiled machine code for the function bodies
    /// This is mapped onto `self.region`.
    functions: Vec<(usize, usize)>,

    /// The computed relocations
    relocations: Relocations,
}

impl Compilation {
    /// Allocates the runtime data structures with the given flags
    pub fn new(module: Module, region: Region, functions: Vec<(usize, usize)>, relocations: Relocations, instance: Instance) -> Self {
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
                let target_func_addr: isize = self.get_function(r.func_index).as_ptr() as isize;
                let body = self.get_function(i);
                unsafe {
                    let reloc_addr = body.as_ptr().offset(r.offset as isize) as isize;
                    let reloc_addend = r.addend as isize - 4;
                    let reloc_delta_i32 = (target_func_addr - reloc_addr + reloc_addend) as i32;
                    write_unaligned(reloc_addr as *mut i32, reloc_delta_i32);
                }
            }
        }
    }

    fn get_function(&self, index: usize) -> &[u8] {
        let region_start = self.region.start().as_u64() as *mut u8;
        let (offset, size) = self.functions[index];
        unsafe {
            slice::from_raw_parts(region_start.add(offset), size)
        }
    } 

    /// Emit a `Code` instance
    pub fn emit(mut self) -> Code {
        self.relocate();

        let vmctx = self.instance.generate_vmctx();

        let start_index = self.module.start_func
            .expect("No start function");
        
        let start_ptr = self.get_function(start_index).as_ptr();

        Code::new(self.module, self.region, self.instance, vmctx, start_ptr)
    }
}

/// Define functions, etc and then "compile"
/// it all into a `Compliation`.
pub struct Compiler<'isa> {
    isa: &'isa TargetIsa,

    contexts: Vec<(cretonne::Context, usize)>,

    total_size: usize,
}

impl<'isa> Compiler<'isa> {
    pub fn new(isa: &'isa TargetIsa) -> Self {
        Self::with_capacity(isa, 0)
    }

    pub fn with_capacity(isa: &'isa TargetIsa, capacity: usize) -> Self {
        Compiler {
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
    pub fn compile(self, module: Module, data_initializers: &[DataInitializer]) -> Result<Compilation> {
        let mut region = sip::allocate_region(self.total_size)
            .ok_or(Error::NO_MEMORY)?;

        let mut functions = Vec::with_capacity(self.contexts.len());
        let mut relocs = Vec::with_capacity(self.contexts.len());

        let mut offset = 0;
        let region_start = region.start().as_u64() as usize;
        
        // emit functions to memory
        for (ref ctx, size) in self.contexts.iter() {
            let mut reloc_sink = RelocSink::new(&ctx.func);
            ctx.emit_to_memory((region_start + offset) as *mut u8, &mut reloc_sink, self.isa);
            functions.push((offset, *size));
            relocs.push(reloc_sink.func_relocs);

            offset += size;
        }

        let instance = Instance::new(&module, data_initializers);

        Ok(Compilation::new(module, region, functions, relocs, instance))
    }
}