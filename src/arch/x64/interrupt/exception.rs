use arch::macros::{interrupt_stack, interrupt_stack_err, interrupt_stack_page};

interrupt_stack!(divide_by_zero, _stack, {
    println!("Divide by zero fault");
});

interrupt_stack!(debug, _stack, {
    println!("Debug trap");
});

interrupt_stack!(non_maskable, _stack, {
    println!("Non-maskable interrupt");
});

interrupt_stack!(breakpoint, _stack, {
    println!("Breakpoint trap");
});

interrupt_stack!(overflow, _stack, {
    println!("Overflow trap");
});

interrupt_stack!(bound_range_exceeded, _stack, {
    println!("Bound Range Exceeded fault");
});

interrupt_stack!(invalid_opcode, _stack, {
    println!("Inavlid Opcode fault");
});

interrupt_stack!(device_not_available, _stack, {
    println!("Device not available fault");
});

interrupt_stack_err!(double_fault, stack, error, {
    println!("Double Fault: {}|{:?}", error, stack);
});

interrupt_stack_err!(invalid_tss, _stack, _error, {
    println!("Invalid TSS fault");
});

interrupt_stack_err!(segment_not_present, stack, error, {
    println!("Segment not present fault");
    println!("{}|{:?}", error, stack);
});

interrupt_stack_err!(stack_segment_fault, _stack, _error, {
    println!("Stack-Segment fault");
});

interrupt_stack_err!(general_protection_fault, _stack, _error, {
    println!("General Protection Fault");
});

interrupt_stack_page!(page_fault, stack, error, {
    println!("Page fault");
    println!("{:?}|{:?}", error, stack);
    let cr2: u64;
    asm!("mov %cr2, $0" : "=r"(cr2));
    println!("Faulting Address: {:#x}", cr2);
    loop {}
    // SCHEDULER.with_current(|ctx| {
    //     if let Some(ref mut mem) = ctx.stack {
    //         let faulting_addr = Cr2::read();
    //         if mem.contains(faulting_addr) {
    //             let page = Page::containing_address(faulting_addr);
    //             mem.map(page);
    //         }
    //     }
    // });
});

interrupt_stack!(x87_floating_point, _stack, {
    println!("x87 Floating-Point Exception");
});

interrupt_stack_err!(alignment_check, _stack, _error, {
    println!("Alignment Check fault");
});

interrupt_stack!(machine_check, _stack, {
    println!("Machine Check abort");
});

interrupt_stack!(simd_floating_point, _stack, {
    println!("SIMD Floating-Point Exception");
});

interrupt_stack!(virtualization, _stack, {
    println!("Virtualization Exception");
});

interrupt_stack_err!(security, _stack, _error, {
    println!("Security Exception");
});

/// He's dead, Jim
interrupt_stack!(triple_fault, _stack, {
    println!("Triple Fault");
});