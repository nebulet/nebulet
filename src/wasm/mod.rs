//! The wasm compiler and runtime
//! Uses Cretonne as the compiler
//! 
//! Some code is taken from wasmstandalone
//! - https://github.com/sunfishcode/wasmstandalone

pub mod runtime;

use cton_wasm::translate_module;
use cton_native;
use self::runtime::{Instance, Module, ModuleEnvironment};
use cretonne::{result::CtonError, isa::TargetIsa};
use cretonne::settings::{self, Configurable};

pub fn wasm_test() {
    let (mut flag_builder, isa_builder) = cton_native::builders()
        .expect("Host machine not supported.");

    flag_builder.set("opt_level", "best").unwrap();

    let isa = isa_builder.finish(settings::Flags::new(&flag_builder));

    let mut module = Module::new();
    let mut environ = ModuleEnvironment::new(isa.flags(), &mut module);
    translate_module(MEMORY_WASM, &mut environ).unwrap();

    let translation = environ.finish_translation();
    println!("Compiling WASM");
    let compliation = translation.compile(&*isa)
        .unwrap();
    println!("WASM compiled!");

    let code = compliation.emit();

    println!("Attempting to execute");

    code.execute();

    println!("It didn't crash!");
}

static CALL_WASM: &'static [u8] = include_bytes!("call.wasm");
static MEMORY_WASM: &'static [u8] = include_bytes!("memory.wasm");