
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
#![feature(integer_atomics)]
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
extern crate hashmap_core;
extern crate nabi;

#[macro_use]
mod arch;
mod panic;
mod memory;
mod time;
mod common;
mod allocator;
mod consts;
mod task;
mod abi;
mod object;

pub use arch::*;
pub use consts::*;

use macros::println;

use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

#[global_allocator]
static ALLOCATOR: allocator::Allocator = allocator::Allocator;

/// The count of all CPUs that can have work scheduled
static CPU_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

extern fn example_thread_entry(arg: usize) -> i32 {
    println!("In example thread: {}", arg);

    0
}

extern fn kernel_thread(_env: usize) -> i32 {
    for i in 0..256 {
        println!("Creating thread: {}", i);
        let thread = task::LockedThread::create(&format!("example thread {}", i), example_thread_entry, i, 16 * 1024)
            .expect("Could not create example thread");
        thread.resume();
    }

    0
}

pub fn kmain(cpus: usize) -> ! {
    CPU_COUNT.store(cpus, Ordering::SeqCst);

    println!("Creating kernel thread");
    let kthread = task::LockedThread::create("[init]", kernel_thread, 0, 16 * 1024)
        .expect("Could not create kernel thread");
    kthread.resume();

    task::resched();

    loop {}
}