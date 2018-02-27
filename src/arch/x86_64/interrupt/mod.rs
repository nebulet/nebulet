//! Interrupt helpers

pub mod exception;
pub mod irq;
pub mod gdt;
use devices::pic;

pub fn enable_interrupts() {
    restore_interrupts((0, 0));
}

/// Disables all interrupts and returns the masks
pub fn disable_interrupts() -> (u8, u8) {
    unsafe {
        disable();

        let saved_masks = (pic::MASTER.data.read(), pic::SLAVE.data.read());

        pic::MASTER.data.write(0xFF);
        pic::SLAVE.data.write(0xFF);

        saved_masks
    }
}

pub fn restore_interrupts(masks: (u8, u8)) {
    unsafe {
        disable();

        pic::MASTER.data.write(masks.0);
        pic::SLAVE.data.write(masks.1);

        enable();
    }
}

/// Disables interrupts, runs the supplied function, and then restores interrupts
/// Taken from LambdaOS, which is taken from Robot Gries' os
pub fn disable_for<F: FnOnce() -> T, T>(f: F) -> T {
    let masks = disable_interrupts();

    let result: T = f();

    restore_interrupts(masks);

    result
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