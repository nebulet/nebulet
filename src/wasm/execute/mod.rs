//! JIT-style runtime for Webassembly

use cretonne::isa::TargetIsa;
use alloc::{String, Vec};
use core::mem;
use core::ptr::write_unaligned;
use wasm::runtime::{Compilation, ModuleTranslation, Relocations, Instance};

/// Executes a module that has been translated with the `standalone::Runtime` runtime implementation
pub fn compile_module<'data, 'module>(
    isa: &TargetIsa,
    translation: &ModuleTranslation<'data, 'module>,
) -> Result<Compilation<'module>, String> {
    let (mut compliation, relocations) = translation.compile(isa)?;

    // let start_index = compliation.module.start_func.ok_or_else(|| {
    //     String::from("No start function defined, aborting execution.")
    // })?;

    // let code_buf = &compliation.functions[start_index];

    // println!("code buffer: ({:#x}: {:#x})", code_buf.as_ptr() as usize, code_buf.len());

    // loop {}

    // Apply relocations
    relocate(&mut compliation, &relocations);

    Ok(compliation)
}

/// Performs the relocations inside the function bytecode, provided the necessary metadata
fn relocate(compilation: &mut Compilation, relocations: &Relocations) {
    // The relocations are relative to the relocation's address plus four bytes
    // TODO: Support architectures other than x86_64, and other reloc kinds.
    for (i, function_relocs) in relocations.iter().enumerate() {
        for ref r in function_relocs {
            let target_func_address: isize = compilation.functions[r.func_index].as_ptr() as isize;
            let body = &mut compilation.functions[i];
            unsafe {
                let reloc_address = body.as_mut_ptr().offset(r.offset as isize ) as isize;
                let reloc_addend = r.addend as isize - 4;
                let reloc_delta_i32 = (target_func_address - reloc_address + reloc_addend) as i32;
                write_unaligned(reloc_address as *mut i32, reloc_delta_i32);
            }
        }
    }
}

/// Create the VmCtx data structure for the JIT'd code to use. This must
/// match the VmCtx layout in the runtime.
fn make_vmctx(instance: &mut Instance) -> Vec<*mut u8> {
    let mut memories = Vec::new();
    let mut vmctx = Vec::new();
    vmctx.push(instance.globals.as_mut_ptr());
    for mem in &mut instance.memories {
        memories.push(mem.as_mut_ptr());
    }
    vmctx.push(memories.as_mut_ptr() as *mut u8);
    vmctx
}

/// Jumps to the region of memory that contains the code and executes the start function of the module.
pub fn execute(compliation: &Compilation, instance: &mut Instance) -> Result<(), String> {
    let start_index = compliation.module.start_func.ok_or_else(|| {
        String::from("No start function defined, aborting execution.")
    })?;

    // TODO: make memory executable
    use arch::paging::ActivePageTable;
    use x86_64::structures::paging::{Page, PageTableFlags};
    use x86_64::VirtAddr;

    let vmctx = make_vmctx(instance);

    let mut active_table = unsafe { ActivePageTable::new() };
    let flags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL | PageTableFlags::WRITABLE;

    for ref func_buf in &compliation.functions {
        let start_page = Page::containing_address(VirtAddr::new(func_buf.as_ptr() as u64));
        let end_page = Page::containing_address(VirtAddr::new(func_buf.as_ptr() as u64 + func_buf.len() as u64 - 1));

        for page in Page::range_inclusive(start_page, end_page) {
            active_table.remap(page, flags).flush(&mut active_table);
        }
    }

    let code_buf = &compliation.functions[start_index];

    // Here, we can just transmute the code buffer to a function
    // with one argument and call it.
    let start_func = unsafe {
        mem::transmute::<_, fn(*const *mut u8)>(code_buf.as_ptr())
    };

    println!("Function count: {}", compliation.functions.len());

    start_func(vmctx.as_ptr());

    // if this returned safely, yay!
    Ok(())
}