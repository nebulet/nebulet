
#![no_std]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(pointer_methods)]
#![feature(thread_local)]
#![no_main]

#[macro_use]
extern crate lazy_static;
extern crate os_bootinfo;
extern crate x86_64;
extern crate spin;
extern crate rlibc;

mod arch;
mod panic;
mod memory;
mod time;

pub use arch::*;

use macros::println;

use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

/// A unique number that identifies the current CPU - used for scheduling
#[thread_local]
static CPU_ID: AtomicUsize = ATOMIC_USIZE_INIT;

/// The count of all CPUs that can have work scheduled
static CPU_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

pub fn kmain(cpus: usize) -> ! {
    CPU_ID.store(0, Ordering::SeqCst);
    CPU_COUNT.store(cpus, Ordering::SeqCst);

    println!("::kmain({})", cpus);

    

    loop {
        unsafe {
            interrupt::halt();
        }
    }
}