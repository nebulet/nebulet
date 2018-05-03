// use sched::{self, Thread};
use arch::interrupt;
use arch::asm::read_gs_offset64;
use task::thread::Thread;
use core::ptr;
use core::sync::atomic::{fence, Ordering};

use x86_64::registers::model_specific::Msr;

static mut CPU0: Cpu = Cpu {
    direct: ptr::null_mut(),
    current_thread: ptr::null_mut(),
    in_irq: 0,
    cpu_id: 0,
    preempt_counter: 0,
    preempt_requested: false,
};

#[repr(C, packed)]
pub struct Cpu {
    // Direct pointer to self
    pub direct: *mut Cpu,

    // The current thread
    pub current_thread: *mut Thread,

    // currently in irq
    pub in_irq: u32,

    // The cpu id (starts at 0)
    pub cpu_id: u32,

    // The preempt counter
    pub preempt_counter: u32,

    pub preempt_requested: bool,
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

// cpu::prempt functions
pub mod preempt {
    use super::*;
    
    #[inline]
    fn requested() -> bool {
        current().preempt_requested
    }

    #[inline]
    pub unsafe fn disable() {
        current().preempt_counter += 1;
        fence(Ordering::SeqCst);
    }

    #[inline]
    pub unsafe fn enable() {
        fence(Ordering::SeqCst);
        current().preempt_counter -= 1;
        if allowed() && requested() && irq::enabled() {
            current().preempt_requested = false;
            // use thread::reschedule;
            // reschedule();
        }
    }

    #[inline]
    /// Request that a preempt occurs
    pub fn request() {
        if allowed() {
            current().preempt_requested = false;
            // use thread::reschedule;
            // unsafe {
            //     reschedule();
            // }
        } else {
            current().preempt_requested = true;
        }
    }

    #[inline]
    pub fn allowed() -> bool {
        current().preempt_counter == 0
    }
}

pub mod thread {
    use super::*;

    #[inline]
    /// This should be safe, because it'll always be called after
    /// a default thread exists in the percpu data structure
    pub fn get() -> &'static mut Thread {
        unsafe { &mut *current().current_thread }
    }

    #[inline]
    /// Definetly unsafe
    pub unsafe fn set(thread: &mut Thread) {
        current().current_thread = thread as *mut _;
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

    #[inline]
    pub fn lock<F>(f: F) where
        F: Fn() {
        let irqs_enabled = enabled();
        if irqs_enabled {
            unsafe { disable(); }
        }

        f();

        if irqs_enabled {
            unsafe { enable(); }
        }
    }
}