//! Interrupt helpers

// pub mod idt;

pub fn init() {
    // idt::init();
}

/// Disable interrupts
#[inline(always)]
pub unsafe fn disable() {
    asm!("cli" : : : : "intel", "volatile");
}

/// Enable interrupts
#[inline(always)]
pub unsafe fn enable() {
    asm!("sti" : : : : "intel", "volatile");
}

/// Enable interrupts and halt
/// This waits for the next interrupt
#[inline(always)]
pub unsafe fn enable_and_halt() {
    asm!("
        sti
        hlt
    " : : : : "intel", "volatile");
}

/// Enable interrupts and nop
/// This allows the IF flag to be processed
/// Use this instead of `enable`
#[inline(always)]
pub unsafe fn enable_and_nop() {
    asm!("
        sti
        nop
    " : : : : "intel", "volatile");
}

/// Halt
#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" : : : : "intel", "volatile");
}

/// Pauses
pub fn pause() {
    unsafe {
        asm!("pause" : : : : "intel", "volatile");
    }
}