
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
#![feature(naked_functions)]
#![no_main]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate os_bootinfo;
extern crate x86_64;
extern crate spin;
extern crate rlibc;
extern crate bit_field;
#[cfg(feature = "linked_alloc")]
extern crate linked_list_allocator;
#[macro_use]
extern crate alloc;

mod arch;
mod panic;
mod memory;
mod time;
mod common;
mod allocator;
mod consts;
mod context;
mod abi;

pub use arch::*;
pub use consts::*;

use macros::println;

use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

#[global_allocator]
static ALLOCATOR: allocator::Allocator = allocator::Allocator;

/// The count of all CPUs that can have work scheduled
static CPU_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

pub fn kmain(cpus: usize) -> ! {
    CPU_COUNT.store(cpus, Ordering::SeqCst);

    context::SCHEDULER.spawn("test context1".into(), example_context).unwrap();
    context::SCHEDULER.spawn("test context2".into(), example_context).unwrap();
    context::SCHEDULER.spawn("test context3".into(), example_context).unwrap();

    loop {
        unsafe {
            interrupt::halt();
        }
    }
}

extern "C" fn example_context() {
    println!("Context running!");
}