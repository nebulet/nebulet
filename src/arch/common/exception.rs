use object::Thread;
use cranelift_codegen::ir::TrapCode;

#[inline]
pub fn page_fault_handler(faulting_addr: *const ()) -> bool {
    let current_thread = Thread::current();

    // {
    //     let stack = &mut current_thread.stack;
    //     println!("faulting addr: {:p}", faulting_addr);

    //     if stack.contains_addr(faulting_addr) {
    //         println!("faulting stack addr: {:p}", faulting_addr);
    //         let _ = stack.region.map_page(faulting_addr);
    //         return;
    //     }
    // }

    if let Some(process) = current_thread.parent() {
        let instance = process.initial_instance();
        let memory = &instance.memories[0];
        
        if likely!(memory.in_mapped_bounds(faulting_addr)) {
            // this path should be as low-latency as possible.
            // just map in the offending page
            let _ = memory.region().map_page(faulting_addr);
            true
        } else if memory.in_unmapped_bounds(faulting_addr) {
            process.handle_trap(TrapCode::HeapOutOfBounds);

            true
        } else {
            false
        }
    } else {
        panic!("Intrinsic thread page faulted at {:p}", faulting_addr);
    }
}

#[inline]
pub fn invalid_opcode_handler(faulting_addr: *const ()) {
    let current_thread = Thread::current();
    if let Some(process) = current_thread.parent() {
        let code = process.code();

        if let Some(trap_code) = code.lookup_trap_code(faulting_addr) {
            process.handle_trap(trap_code);
        }
    }
}