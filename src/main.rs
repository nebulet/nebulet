
#![no_std]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(pointer_methods)]
#![feature(thread_local)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![feature(global_asm)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![feature(compiler_builtins_lib)]
#![feature(nonnull_cast)]
#![feature(repr_transparent)]
#![feature(box_into_raw_non_null)]
#![feature(box_syntax)]
#![feature(unsize, coerce_unsized)]
#![feature(dropck_eyepatch)]
#![feature(arbitrary_self_types)]
#![feature(nll)]
#![feature(fnbox)]

#![no_main]
#![deny(warnings)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate os_bootinfo;
extern crate x86_64;
extern crate spin;
extern crate rlibc;
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
extern crate kernel_ref_derive;

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

pub use consts::*;

#[global_allocator]
pub static ALLOCATOR: allocator::Allocator = allocator::Allocator;

pub fn kmain() -> ! {
    println!("Nebulet v{}", VERSION);

    use object::{ThreadRef, ProcessRef, CodeRef};

    let thread = ThreadRef::new(1024 * 1024, || {
        let code = CodeRef::compile(include_bytes!("wasm/wasmtests/exit.wasm"))
            .unwrap();
        for i in 0..10 {
            let process = ProcessRef::create(format!("test-process[{}]", i), code.clone())
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
