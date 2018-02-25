
use x86_64::structures::idt::Idt;
use interrupt::*;
use devices::pic;
use interrupt;

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.breakpoint.set_handler_fn(exception::breakpoint);

        idt[33].set_handler_fn(irq::keyboard);

        idt
    };
}

pub fn init() {
    IDT.load();

    unsafe {
        interrupt::enable_and_nop();
    }
}