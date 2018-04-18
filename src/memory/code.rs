use x86_64::structures::paging::PageTableFlags;
use core::mem;

use wasm::runtime::instance::{Instance, VmCtx};
use wasm::runtime::Module;
use memory::Region;

use nabi::{Result, Error};

/// Represents the area of memory that contains compiled code
pub struct Code {
    module: Module,
    instance: Instance,
    region: Region,
    start_func: *const u8,
}

impl Code {
    pub fn new(module: Module, mut region: Region, instance: Instance, start_func: *const u8) -> Result<Self> {
        let flags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL;
        region.remap(flags)?;

        Ok(Code {
            module,
            instance,
            region,
            start_func,
        })
    }
    
    pub fn execute(&mut self) {
        let start_func = unsafe {
            mem::transmute::<_, fn(*const VmCtx)>(self.start_func)
        };

        let vmctx = self.instance.generate_vmctx();

        start_func(&vmctx as *const _);
    }
}
