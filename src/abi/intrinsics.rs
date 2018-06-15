use object::ProcessRef;
use wasm::instance::VmCtx;

/// `count` is the number of wasm pages to grow the memory by.
pub extern fn grow_memory(count: u32, vmctx: &VmCtx) -> i32 {
    let data = vmctx.data();
    let process: &ProcessRef = &*data.process;
    let mut instance = process.instance().write();
    let memory = &mut instance.memories[0];

    if let Ok(old_count) = memory.grow(count as usize) {
        old_count as i32
    } else {
        -1
    }
}

pub extern fn current_memory(vmctx: &VmCtx) -> u32 {
    let data = vmctx.data();
    let process: &ProcessRef = &*data.process;
    let instance = process.instance().read();
    let memory = &instance.memories[0];

    memory.page_count() as u32
}

pub extern fn debug_addr(addr: *const (), _: &VmCtx) {
    println!("addr: {:p}", addr);
}
