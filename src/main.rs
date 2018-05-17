
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
pub mod process;

pub use consts::*;

#[global_allocator]
pub static ALLOCATOR: allocator::Allocator = allocator::Allocator;

pub fn kmain() -> ! {
    println!("Nebulet v{}", VERSION);
    
    // tests::test_all();

    use task::Thread;

    for i in 0..1 {
        let thread = Thread::new(1024 * 1024, test_thread, i).unwrap();
        thread.resume().unwrap();
    }

    unsafe {
        ::arch::interrupt::enable();
        loop {
            ::arch::interrupt::halt();
        }
    }
}

extern fn test_thread(arg: usize) {
    println!("thread: {}", arg);

    use process::Process;
    use object::{GlobalHandleTable, HandleRights};
    let process = Process::compile("abi-test process", include_bytes!("wasm/wasmtests/exit.wasm"))
        .unwrap();

    let proc_index = GlobalHandleTable::get_mut()
        .allocate(process, HandleRights::all())
        .unwrap();
    
    let handle_table = GlobalHandleTable::get();
    {
        let mut proc_handle = handle_table
            .get_handle(proc_index)
            .unwrap()
            .lock_cast::<Process>()
            .unwrap();

        proc_handle.start(proc_index).unwrap();
    }
}
