use macros::{println, interrupt_stack, interrupt_stack_err, interrupt_stack_page};

interrupt_stack!(divide_by_zero, stack, {
    println!("Divide by zero fault");
});

interrupt_stack!(debug, stack, {
    println!("Debug trap");
});

interrupt_stack!(non_maskable, stack, {
    println!("Non-maskable interrupt");
});

interrupt_stack!(breakpoint, stack, {
    println!("Breakpoint trap");
});

interrupt_stack!(overflow, stack, {
    println!("Overflow trap");
});

interrupt_stack!(bound_range_exceeded, stack, {
    println!("Bound Range Exceeded fault");
});

interrupt_stack!(invalid_opcode, stack, {
    println!("Inavlid Opcode fault");
});

interrupt_stack!(device_not_available, stack, {
    println!("Device not available fault");
});

interrupt_stack_err!(double_fault, stack, error, {
    println!("Double Fault: {}|{:?}", error, stack);
});

interrupt_stack_err!(invalid_tss, stack, error, {
    println!("Invalid TSS fault");
});

interrupt_stack_err!(segment_not_present, stack, error, {
    println!("Segment not present fault");
    println!("{}|{:?}", error, stack);
});

interrupt_stack_err!(stack_segment_fault, stack, error, {
    println!("Stack-Segment fault");
});

interrupt_stack_err!(general_protection_fault, stack, error, {
    println!("General Protection Fault");
});

interrupt_stack_page!(page_fault, stack, error, {
    println!("Page fault");
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

interrupt_stack!(x87_floating_point, stack, {
    println!("x87 Floating-Point Exception");
});

interrupt_stack_err!(alignment_check, stack, error, {
    println!("Alignment Check fault");
});

interrupt_stack!(machine_check, stack, {
    println!("Machine Check abort");
});

interrupt_stack!(simd_floating_point, stack, {
    println!("SIMD Floating-Point Exception");
});

interrupt_stack!(virtualization, stack, {
    println!("Virtualization Exception");
});

interrupt_stack_err!(security, stack, error, {
    println!("Security Exception");
});

/// All is over, jim is dead
interrupt_stack!(triple_fault, stack, {
    println!("Triple Fault");
});