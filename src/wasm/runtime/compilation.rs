//! A `Compilation` contains the compiled function bodies for a WebAssembly
//! module

use super::module::Module;
use super::{Relocations, DataInitializer};
use cretonne_codegen::{self, isa::TargetIsa, binemit::Reloc};
use super::RelocSink;
use super::abi::ABI_MAP;

use memory::{Code, Region, sip};

use nabi::{Result, Error};

use core::ptr::write_unaligned;
use alloc::{Vec, String};

#[derive(Debug)]
enum FunctionType {
    Local {
        offset: usize,
        size: usize,
    },
    External {
        module: String,
        name: String,
    }
}

#[derive(Debug)]
pub struct Compilation {
    region: Region,

    /// Compiled machine code for the function bodies
    /// This is mapped onto `self.region`.
    functions: Vec<FunctionType>,

    first_local_function: usize,

    /// The computed relocations
    relocations: Relocations,
}

impl Compilation {
    /// Allocates the runtime data structures with the given flags
    fn new(region: Region, functions: Vec<FunctionType>, relocations: Relocations) -> Self {
        let first_local_function = functions
            .iter()
            .position(|f| match f {
                FunctionType::Local {..} => true,
                _ => false,
            }).unwrap();

        Compilation {
            region,
            functions,
            first_local_function,
            relocations,
        }
    }

    /// Relocate the compliation.
    fn relocate(&mut self, module: &Module) -> Result<()> {
        // The relocations are absolute addresses
        // TODO: Support architectures other than x86_64, and other reloc kinds.
        for (i, function_relocs) in self.relocations.iter().enumerate() {
            for ref r in function_relocs {
                let (target_func_addr, _is_local) = self.get_function_addr(module, r.func_index)?;
                let body_addr = self.get_function_addr(module, i + self.first_local_function)?.0;
                let reloc_addr = unsafe{ body_addr.offset(r.offset as isize) };

                match r.reloc {
                    Reloc::Abs8 => {
                        unsafe {
                            write_unaligned(reloc_addr as *mut usize, target_func_addr as usize)
                        };
                    }
                    _ => unimplemented!()
                }
            }
        }

        Ok(())
    }

    fn get_function_addr(&self, module_ref: &Module, index: usize) -> Result<(*const u8, bool)> {
        match self.functions[index] {
            FunctionType::Local {
                offset,
                size: _,
            } => {
                Ok(((self.region.start().as_u64() as usize + offset) as *const u8, true))
            },
            FunctionType::External {
                ref module,
                ref name,
            } => {
                match module.as_str() {
                    "abi" => {
                        let abi_func = ABI_MAP.get(name.as_str())?;

                        let imported_sig = &module_ref.signatures[index];

                        if abi_func.same_sig(imported_sig) {
                            Ok((abi_func.ptr, false))
                        } else {
                            println!("Incorrect signature");
                            println!("ABI sig: {:?}", abi_func);
                            println!("Import sig: {:?}", imported_sig);
                            Err(Error::INTERNAL)
                        }
                    },
                    _ => Err(Error::INTERNAL),
                }
            },
        }
    }

    /// Emit a `Code` instance
    pub fn emit(mut self, module: Module, data_initializers: Vec<DataInitializer>) -> Result<Code> {
        self.relocate(&module)?;

        let start_index = module.start_func?;
        let start_ptr = self.get_function_addr(&module, start_index)?.0;

        Code::new(module, data_initializers, self.region, start_ptr)
    }
}

/// Define functions, etc and then "compile"
/// it all into a `Compliation`.
pub struct Compiler<'isa> {
    isa: &'isa TargetIsa,

    contexts: Vec<(cretonne_codegen::Context, usize)>,

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
    pub fn define_function(&mut self, mut ctx: cretonne_codegen::Context) -> Result<()> {
        let code_size = ctx.compile(self.isa)
            .map_err(|e| {
                println!("Compile error: {:?}", e);
                Error::INTERNAL
            })? as usize;

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
    pub fn compile(self, module: &Module) -> Result<Compilation> {
        let region = sip::allocate_region(self.total_size)
            .ok_or(Error::NO_MEMORY)?;

        let mut functions = Vec::with_capacity(module.functions.len());
        let mut relocs = Vec::with_capacity(self.contexts.len());

        let mut offset = 0;
        let region_start = region.start().as_u64() as usize;

        for (module, name) in module.imported_funcs.iter().cloned() {
            functions.push(FunctionType::External {
                module,
                name,
            });
        }

        // emit functions to memory
        for (ref ctx, size) in self.contexts.iter() {
            // TODO(gmorenz): We probably want traps?
            use cretonne_codegen::binemit::NullTrapSink;

            let mut reloc_sink = RelocSink::new();
            unsafe {
                ctx.emit_to_memory(self.isa, (region_start + offset) as *mut u8, &mut reloc_sink, &mut NullTrapSink {});
            }
            functions.push(FunctionType::Local {
                offset,
                size: *size,
            });
            relocs.push(reloc_sink.func_relocs);

            offset += size;
        }

        Ok(Compilation::new(region, functions, relocs))
    }
}
