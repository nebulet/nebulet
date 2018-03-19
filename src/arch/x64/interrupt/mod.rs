//! Interrupt helpers

pub mod exception;
pub mod irq;
pub mod gdt;

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