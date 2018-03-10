
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

    if (arg > 0) {
        println!("Creating thread");
        let thread = task::LockedThread::create(&format!("test thread {}", arg), example_thread_entry, arg - 1, 4096 * 4)
            .expect("Could not create example thread");

        thread.resume();
    } else {
        println!("Finished creating threads");
    }

    0
}

pub fn kmain(cpus: usize) -> ! {
    CPU_COUNT.store(cpus, Ordering::SeqCst);

    let thread = task::LockedThread::create("test thread 1", example_thread_entry, 10, 4096 * 4)
        .expect("Could not create example thread");

    thread.resume();

    task::resched();

    unsafe {
        interrupt::enable_and_nop();
    }
    loop {}
}

extern fn example_context1() {
    println!("Context 1 running!");
    loop {}
}

extern fn example_context2() {
    println!("Context 2 running");
    // loop {}
}

extern fn example_context3() {
    loop {
        println!("Context 3 running");
    }
}