//! The wasm compiler and runtime
//! Uses Cretonne as the compiler
//! 
//! Some code is taken from wasmstandalone
//! - https://github.com/sunfishcode/wasmstandalone

pub mod runtime;
pub mod execute;

use cton_wasm::translate_module;
use cton_native;
use self::execute::{compile_module, execute};
use self::runtime::{Instance, Module, ModuleEnvironment};
use cretonne::isa::TargetIsa;
use cretonne::settings::{self, Configurable};

pub fn wasm_test() {
    let (mut flag_builder, isa_builder) = cton_native::builders().unwrap_or_else(|_| {
        panic!("Host machine not supported!");
    });

    let isa = isa_builder.finish(settings::Flags::new(&flag_builder));

    let mut module = Module::new();
    let mut environ = ModuleEnvironment::new(isa.flags(), &mut module);
    translate_module(CALL_WASM, &mut environ).unwrap();

    let translation = environ.finish_translation();
    println!("Compiling WASM");
    let compliation = compile_module(&*isa, &translation).unwrap();
    println!("WASM compiled!");
    let mut instance = Instance::new(compliation.module, &translation.lazy.data_initializers);

    println!("Attempting to execute");
    // Here it goes!
    execute(&compliation, &mut instance).unwrap();

    println!("It didn't crash!");
}

static CALL_WASM: &'static [u8] = include_bytes!("call.wasm");