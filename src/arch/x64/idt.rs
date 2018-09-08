use arch::devices::pic;
use arch::interrupt::{self, *};
use core::ptr;
use spin::Once;
use x86_64::structures::idt::{ExceptionStackFrame, InterruptDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;

pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<gdt::Gdt> = Once::new();

pub fn default_idt() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();
    idt.divide_by_zero.set_handler_fn(exception::divide_by_zero);
    idt.debug.set_handler_fn(exception::debug);
    idt.non_maskable_interrupt
        .set_handler_fn(exception::non_maskable);
    idt.breakpoint.set_handler_fn(exception::breakpoint);
    idt.overflow.set_handler_fn(exception::overflow);
    idt.bound_range_exceeded
        .set_handler_fn(exception::bound_range_exceeded);
    idt.invalid_opcode.set_handler_fn(exception::invalid_opcode);
    idt.device_not_available
        .set_handler_fn(exception::device_not_available);
    idt.double_fault.set_handler_fn(exception::double_fault);
    idt.invalid_tss.set_handler_fn(exception::invalid_tss);
    idt.segment_not_present
        .set_handler_fn(exception::segment_not_present);
    idt.stack_segment_fault
        .set_handler_fn(exception::stack_segment_fault);
    idt.general_protection_fault
        .set_handler_fn(exception::general_protection_fault);
    idt.page_fault.set_handler_fn(exception::page_fault);
    idt.x87_floating_point
        .set_handler_fn(exception::x87_floating_point);
    idt.alignment_check
        .set_handler_fn(exception::alignment_check);
    idt.machine_check.set_handler_fn(exception::machine_check);
    idt.simd_floating_point
        .set_handler_fn(exception::simd_floating_point);
    idt.virtualization.set_handler_fn(exception::virtualization);
    idt.security_exception.set_handler_fn(exception::security);

    idt[32].set_handler_fn(irq::pit);

    // idt[40].set_handler_fn(irq::rtc);

    idt
}

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;
    use x86_64::structures::gdt::SegmentSelector;

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

pub type Handler = fn(*const ());

static mut EVENT_TABLE: [(Option<Handler>, *const ()); 16] = [(None, ptr::null()); 16];

macro_rules! idt_handlers {
    ($($name:ident ( $value:expr) ),*) => {
        [$( {
                extern "x86-interrupt" fn $name(_: &mut ExceptionStackFrame) {
                    let irq = $value - pic::MASTER_OFFSET;

                    let (handler, arg) = unsafe { EVENT_TABLE[irq as usize] };
                    if let Some(func) = handler {
                        func(arg);
                    }
                    unsafe {
                        if irq < 16 {
                            if irq >= 8 {
                                pic::MASTER.ack();
                                pic::SLAVE.ack();
                            } else {
                                pic::MASTER.ack();
                            }
                        }
                    }
                }
                $name
            }
         ),*
        ]
    }
}

static IDT_HANDLER: [extern "x86-interrupt" fn(&mut ExceptionStackFrame); 16] = idt_handlers! {
    idt_handler_0x20 ( 0x20 ),
    idt_handler_0x21 ( 0x21 ),
    idt_handler_0x22 ( 0x22 ),
    idt_handler_0x23 ( 0x23 ),
    idt_handler_0x24 ( 0x24 ),
    idt_handler_0x25 ( 0x25 ),
    idt_handler_0x26 ( 0x26 ),
    idt_handler_0x27 ( 0x27 ),
    idt_handler_0x28 ( 0x28 ),
    idt_handler_0x29 ( 0x29 ),
    idt_handler_0x2a ( 0x2a ),
    idt_handler_0x2b ( 0x2b ),
    idt_handler_0x2c ( 0x2c ),
    idt_handler_0x2d ( 0x2d ),
    idt_handler_0x2e ( 0x2e ),
    idt_handler_0x2f ( 0x2f )
};

pub fn register_handler(irq: u8, handler: fn(arg: *const ()), arg: *const ()) {
    let index = (irq - pic::MASTER_OFFSET) as usize;

    let int_handler = IDT_HANDLER[index];

    unsafe {
        interrupt::mask(irq);
        EVENT_TABLE[index] = (Some(handler), arg);
        IDT[irq as usize].set_handler_fn(int_handler);
        interrupt::unmask(irq);
    }
}

pub fn unregister_handler(irq: u8) {
    let index = (irq - pic::MASTER_OFFSET) as usize;
    unsafe {
        interrupt::mask(irq);
        EVENT_TABLE[index] = (None, ptr::null());
    }
}
