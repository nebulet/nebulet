use wasm::instance::VmCtx;

/// `count` is the number of wasm pages to grow the memory by.
pub extern "C" fn grow_memory(count: u32, vmctx: &VmCtx) -> i32 {
    let instance = &vmctx.data().user_data.instance;
    let memory = &instance.memories[0];

    if let Ok(old_count) = memory.grow(count as usize) {
        old_count as i32
    } else {
        -1
    }
}

pub extern "C" fn current_memory(vmctx: &VmCtx) -> u32 {
    let instance = &vmctx.data().user_data.instance;
    let memory = &instance.memories[0];

    memory.page_count() as u32
}

pub extern "C" fn debug_addr(addr: *const (), _: &VmCtx) {
    println!("addr: {:p}", addr);
}
