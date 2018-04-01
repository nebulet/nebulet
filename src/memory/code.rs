
use x86_64::VirtAddr;
use x86_64::structures::paging::{PAGE_SIZE, PageTableFlags};
use core::mem;
use alloc::Vec;

use wasm::runtime::instance::{Instance, VmCtx};
use wasm::runtime::Module;
use memory::Region;

/// Represents the area of memory that contains compiled code
pub struct Code {
    module: Module,
    instance: Instance,
    region: Region,
    start_func: *const u8,
    vmctx: VmCtx,
}

impl Code {
    pub fn new(module: Module, mut region: Region, instance: Instance, vmctx: VmCtx, start_func: *const u8) -> Self {
        let flags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL;
        region.remap(flags);

        Code {
            module,
            instance,
            region,
            start_func,
            vmctx,
        }
    }

    pub fn execute(&self) {
        let start_func = unsafe {
            mem::transmute::<_, fn(*const VmCtx)>(self.start_func)
        };

        start_func(&self.vmctx as *const _);
    }
}