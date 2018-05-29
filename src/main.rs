
#![no_std]
#![feature(
    lang_items,
    abi_x86_interrupt,
    asm,
    const_fn,
    decl_macro,
    pointer_methods,
    thread_local,
    alloc,
    allocator_api,
    global_allocator,
    global_asm,
    core_intrinsics,
    naked_functions,
    compiler_builtins_lib,
    nonnull_cast,
    repr_transparent,
    box_into_raw_non_null,
    box_syntax,
    unsize,
    coerce_unsized,
    dropck_eyepatch,
    arbitrary_self_types,
    nll,
    fnbox,
    proc_macro,
    integer_atomics
)]

#![no_main]
#![deny(warnings)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate os_bootinfo;
extern crate x86_64;
extern crate spin;
extern crate bit_field;
#[macro_use]
extern crate alloc;
extern crate hashmap_core;
extern crate nabi;

extern crate cretonne_wasm;
extern crate cretonne_native;
extern crate cretonne_codegen;
extern crate wasmparser;
extern crate nil;
#[macro_use]
extern crate nebulet_derive;

#[macro_use]
pub mod arch;
pub mod panic;
pub mod memory;
pub mod time;
pub mod common;
pub mod allocator;
pub mod consts;
pub mod abi;
pub mod object;
pub mod task;
pub mod wasm;
pub mod externs;

pub use consts::*;

#[global_allocator]
pub static ALLOCATOR: allocator::Allocator = allocator::Allocator;

pub fn kmain() -> ! {
    println!("Nebulet v{}", VERSION);

    use object::{ThreadRef, ProcessRef, CodeRef};

    let thread = ThreadRef::new(1024 * 1024, move || {
        let code = CodeRef::compile(include_bytes!("wasm/wasmtests/exit.wasm"))
            .unwrap();
        for _ in 0..10 {
            let process = ProcessRef::create(code.clone())
                .unwrap();

            process.start().unwrap();
        }
    }).unwrap();

    thread.resume().unwrap();

    unsafe {
        arch::cpu::Local::current()
            .scheduler
            .switch();
    }

    unimplemented!("Arrived back in `kmain` somehow.");
}
