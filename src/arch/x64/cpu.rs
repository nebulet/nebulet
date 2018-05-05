// use sched::{self, Thread};
use arch::interrupt;
use arch::asm::read_gs_offset64;
use core::ptr;

use x86_64::registers::model_specific::Msr;

static mut CPU0: Cpu = Cpu {
    direct: ptr::null_mut(),
    cpu_id: 0,
};

#[repr(C, packed)]
pub struct Cpu {
    // Direct pointer to self
    pub direct: *mut Cpu,

    // The cpu id (starts at 0)
    pub cpu_id: u32,
}

pub unsafe fn init() {
    CPU0.direct = &mut CPU0 as *mut Cpu;

    Msr::new(0xC0000101)
        .write(CPU0.direct as u64);
}

/// cpu functions
#[inline]
pub fn current() -> &'static mut Cpu {
    unsafe {
        &mut *(read_gs_offset64!(0x0) as *mut Cpu)
    }
}

/// cpu::irq functions
pub mod irq {
    use super::*;

    #[inline]
    pub unsafe fn disable() {
        interrupt::disable();
    }

    #[inline]
    pub unsafe fn enable() {
        interrupt::enable();
    }

    #[inline]
    #[must_use]
    pub fn enabled() -> bool {
        let rflags: u64;
        unsafe {
            asm!("pushfq; pop $0" : "=r"(rflags) : : "memory" : "intel", "volatile");
        }
        rflags & (1 << 9) == 1
    }
}