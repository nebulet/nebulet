use x86_64::structures::paging::PageTableFlags;
use core::mem;
use alloc::Vec;

use wasm::runtime::instance::{Instance, VmCtx};
use wasm::runtime::{Module, DataInitializer};
use memory::Region;

use nabi::Result;

/// Represents the area of memory that contains compiled code
pub struct Code {
    data_initializers: Vec<DataInitializer>,
    module: Module,
    region: Region,
    start_func: fn(*const VmCtx),
}

impl Code {
    pub fn new(module: Module, data_initializers: Vec<DataInitializer>, mut region: Region, start_func: *const u8) -> Result<Self> {
        let flags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL;
        region.remap(flags)?;

        assert!(region.contains(start_func as usize));

        let start_func = unsafe {
            mem::transmute::<_, fn(*const VmCtx)>(start_func)
        };

        Ok(Code {
            data_initializers,
            module,
            region,
            start_func,
        })
    }

    pub fn generate_instance(&self) -> Instance {
        Instance::new(&self.module, &self.data_initializers)
    }
    
    pub fn execute(&mut self) {
        let mut instance = self.generate_instance();
        let vmctx = instance.generate_vmctx();

        println!("vmctx: {:#x}", &vmctx as *const _ as usize);

        (self.start_func)(&vmctx as *const _);
    }
}
