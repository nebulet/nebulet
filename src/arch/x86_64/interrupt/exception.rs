use macros::{println, interrupt_stack};

interrupt_stack!(breakpoint, stack, {
    println!("Breakpoint trap");
    println!("{:?}", stack);
});