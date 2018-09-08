//! Interrupt helpers

pub mod exception;
pub mod gdt;
pub mod irq;
use arch::devices::pic;
use arch::idt;

/// Disable interrupts
#[inline(always)]
pub unsafe fn disable() {
    asm!("cli" : : : : "intel", "volatile");
}

/// Enable interrupts and nop
/// This allows the IF flag to be processed
#[inline(always)]
pub unsafe fn enable() {
    asm!("sti
          nop
    " : : : : "intel", "volatile");
}

/// Halt
#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" : : : : "intel", "volatile");
}

/// Pauses
/// This is safe
pub fn pause() {
    unsafe {
        asm!("pause" : : : : "intel", "volatile");
    }
}

#[inline]
pub unsafe fn mask(irq: u8) {
    let irq = irq - pic::MASTER_OFFSET;
    if irq < 8 {
        pic::MASTER.mask_set(irq);
    } else {
        pic::SLAVE.mask_set(irq - 8);
    }
}

#[inline]
pub unsafe fn unmask(irq: u8) {
    let irq = irq - pic::MASTER_OFFSET;
    if irq < 8 {
        pic::MASTER.mask_clear(irq);
    } else {
        pic::SLAVE.mask_clear(irq - 8);
    }
}

pub unsafe fn register_handler(vector: u32, handler: fn(*const ()), arg: *const ()) -> bool {
    if vector > u8::max_value() as _ {
        false
    } else {
        idt::register_handler(vector as u8, handler, arg);

        true
    }
}

pub unsafe fn unregister_handler(vector: u32) -> bool {
    if vector > u8::max_value() as _ {
        false
    } else {
        idt::unregister_handler(vector as u8);

        true
    }
}
