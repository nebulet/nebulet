
#[macro_use]
#[allow(unused_macros)]
pub mod macros;

pub mod start;

pub mod devices;

pub mod interrupt;

#[cfg(feature = "vga")]
pub mod printer;

pub mod idt;

pub mod paging;

pub mod asm;

pub mod cpu;

pub mod lock;

pub mod context;

pub mod memory;

pub mod pci;

global_asm!(include_str!("routines.asm"));
