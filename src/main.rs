
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
    global_asm,
    core_intrinsics,
    naked_functions,
    compiler_builtins_lib,
    nonnull_cast,
    box_into_raw_non_null,
    box_syntax,
    unsize,
    coerce_unsized,
    dropck_eyepatch,
    arbitrary_self_types,
    nll,
    fnbox,
    proc_macro,
    integer_atomics,
    platform_intrinsics,
    panic_implementation,
    range_contains,
    iterator_step_by,
    future_atomic_orderings,
    stmt_expr_attributes,
    get_type_id,
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
#[macro_use]
extern crate nabi;
extern crate raw_cpuid;
extern crate rand_core;
extern crate rand;

extern crate cretonne_wasm;
extern crate cretonne_native;
extern crate cretonne_codegen;
extern crate target_lexicon;
extern crate wasmparser;
extern crate nebulet_derive;

pub mod nil;
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
pub mod sync;
pub mod signals;
pub mod event;

pub use consts::*;

#[global_allocator]
pub static ALLOCATOR: allocator::Allocator = allocator::Allocator;

pub fn kmain(init_wasm: &[u8]) -> ! {
    println!("------------");
    println!("Nebulet v{}", VERSION);

    use object::{Process, Wasm};

    let code = Wasm::compile(init_wasm)
        .unwrap();

    let process = Process::create(code.copy_ref())
        .unwrap();

    process.start().unwrap();

    unsafe {
        arch::cpu::Local::context_switch();
    }

    unreachable!();
}
