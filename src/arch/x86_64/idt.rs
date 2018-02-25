
use x86_64::structures::idt::Idt;
use interrupt::*;
use interrupt;

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.divide_by_zero.set_handler_fn(exception::divide_by_zero);
        idt.debug.set_handler_fn(exception::debug);
        idt.non_maskable_interrupt.set_handler_fn(exception::non_maskable);
        idt.breakpoint.set_handler_fn(exception::breakpoint);
        idt.overflow.set_handler_fn(exception::overflow);
        idt.bound_range_exceeded.set_handler_fn(exception::bound_range_exceeded);
        idt.invalid_opcode.set_handler_fn(exception::invalid_opcode);
        idt.device_not_available.set_handler_fn(exception::device_not_available);
        idt.double_fault.set_handler_fn(exception::double_fault);
        idt.invalid_tss.set_handler_fn(exception::invalid_tss);
        idt.segment_not_present.set_handler_fn(exception::segment_not_present);
        idt.stack_segment_fault.set_handler_fn(exception::stack_segment_fault);
        idt.general_protection_fault.set_handler_fn(exception::general_protection_fault);
        idt.page_fault.set_handler_fn(exception::page_fault);
        idt.x87_floating_point.set_handler_fn(exception::x87_floating_point);
        idt.alignment_check.set_handler_fn(exception::alignment_check);
        idt.machine_check.set_handler_fn(exception::machine_check);
        idt.simd_floating_point.set_handler_fn(exception::simd_floating_point);
        idt.virtualization.set_handler_fn(exception::virtualization);
        idt.security_exception.set_handler_fn(exception::security);

        idt[32].set_handler_fn(irq::pit);
        idt[33].set_handler_fn(irq::keyboard);

        idt[40].set_handler_fn(irq::rtc);

        idt
    };
}

pub fn init() {
    IDT.load();

    unsafe {
        // for now disable interrupts
        // TODO: Enable again after memory works
        interrupt::enable_and_nop();
    }
}