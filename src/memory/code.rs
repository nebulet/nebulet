use x86_64::structures::paging::PageTableFlags;
use core::mem;

use wasm::runtime::instance::{Instance, VmCtx};
use wasm::runtime::Module;
use memory::Region;

use nabi::Result;

/// Represents the area of memory that contains compiled code
pub struct Code {
    module: Module,
    instance: Instance,
    region: Region,
    start_func: fn(*const VmCtx),
}

impl Code {
    pub fn new(module: Module, mut region: Region, instance: Instance, start_func: *const u8) -> Result<Self> {
        let flags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL;
        region.remap(flags)?;

        assert!(region.contains(start_func as usize));

        let start_func = unsafe {
            mem::transmute::<_, fn(*const VmCtx)>(start_func)
        };

        Ok(Code {
            module,
            instance,
            region,
            start_func,
        })
    }
    
    pub fn execute(&mut self) {
        let vmctx = self.instance.generate_vmctx();

        (self.start_func)(&vmctx as *const _);
    }
}
