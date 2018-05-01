
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

#![no_main]
// #![deny(warnings)]

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

use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

#[global_allocator]
pub static ALLOCATOR: allocator::Allocator = allocator::Allocator;

/// The count of all CPUs that can have work scheduled
static CPU_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

pub fn kmain(cpus: usize) -> ! {
    CPU_COUNT.store(cpus, Ordering::SeqCst);

    println!("Nebulet v{}", VERSION);

    // wasm::wasm_test();

    use task::thread::Thread;

    let thread = Thread::new(1024 * 16, test_thread).unwrap();
    let mut idle_thread = Thread::new(256, idle_thread).unwrap();

    unsafe {
        idle_thread.switch_to(&thread);
    }

    loop {
        unsafe { arch::interrupt::halt(); }
    }
}

extern fn test_thread() {
    println!("From thread");

    loop {}
}

extern fn idle_thread() {
    loop {}
}