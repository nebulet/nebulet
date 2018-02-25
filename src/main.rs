
#![no_std]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(pointer_methods)]
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

pub use arch::*;