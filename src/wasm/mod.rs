//! The wasm compiler and runtime
//! Uses Cretonne as the compiler
//!
//! Some code is taken from wasmstandalone
//! - https://github.com/sunfishcode/wasmstandalone

pub mod runtime;

use cretonne_wasm::translate_module;
use cretonne_native;
use self::runtime::{Module, ModuleEnvironment};
use cretonne_codegen::settings::{self, Configurable};
use memory::Code;

use nabi::{Result, Error};

use alloc::Vec;

pub fn compile_module(wasm: &[u8]) -> Result<Code> {
    let (mut flag_builder, isa_builder) = cretonne_native::builders()
        .map_err(|_| Error::INTERNAL)?;

    flag_builder.set("opt_level", "best")
        .map_err(|_| Error::INTERNAL)?;

    let isa = isa_builder.finish(settings::Flags::new(flag_builder));

    let module = Module::new();
    let mut environ = ModuleEnvironment::new(isa.flags(), module);

    translate_module(wasm, &mut environ)
        .map_err(|_| Error::INTERNAL)?;

    let translation = environ.finish_translation();
    let (compliation, module, data_initializers) = translation.compile(&*isa)?;
    
    compliation.emit(module, data_initializers)
}