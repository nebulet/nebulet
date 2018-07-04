use core::ops::{Deref, DerefMut};
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::tss::TaskStateSegment;
use arch::interrupt::{*, self};
use spin::Once;

pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<gdt::Gdt> = Once::new();

pub fn default_idt() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();
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
    // idt[33].set_handler_fn(irq::keyboard);

    // idt[40].set_handler_fn(irq::rtc);

    idt
}

pub fn init() {
    use x86_64::structures::gdt::SegmentSelector;
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    let tss = TSS.call_once(|| {
        let tss = TaskStateSegment::new();
        // tss.interrupt.stack_table[DOUBLE_FAULT_IST_INDEX] = VirtAddr::new()
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = gdt::Gdt::new();
        code_selector = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        tss_selector = gdt.add_entry(gdt::Descriptor::tss_segment(&tss));
        gdt
    });

    // load the new GDT
    gdt.load();

    unsafe {
        // reload code segment register
        set_cs(code_selector);
        // load TSS
        load_tss(tss_selector);
        // load IDT
        IDT = default_idt();
        IDT.load();
    }
}

pub struct IdtGuard {
    inner: &'static mut InterruptDescriptorTable
}

impl <'a> Deref for IdtGuard {
    type Target=InterruptDescriptorTable;
    fn deref(&self) -> &InterruptDescriptorTable {
        self.inner
    }
}

impl <'a> DerefMut for IdtGuard {
    fn deref_mut(&mut self) -> &mut InterruptDescriptorTable {
        self.inner
    }
}

impl IdtGuard {
    /// It is only safe to call this function if interrupts can be enabled when
    /// you drop the result of the call... ugh.
    pub unsafe fn new() -> IdtGuard {
        interrupt::disable();
        IdtGuard {
            inner: &mut IDT
        }
    }
}

impl Drop for IdtGuard {
    fn drop(&mut self) {
        unsafe {
            interrupt::enable();
        }
    }
}
