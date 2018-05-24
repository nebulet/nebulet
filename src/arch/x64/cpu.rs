use x86_64::registers::model_specific::Msr;
use x86_64::registers::flags::Flags;
use core::ptr::NonNull;

use arch::interrupt;
use arch::asm::read_gs_offset64;

use task::{Thread, State, scheduler::Scheduler};

use alloc::boxed::Box;
// use alloc::Vec;

// static GLOBAL: Once<Global> = Once::new();

pub type CpuId = u32;

pub struct Cpu {
    /// The cpu id (starts at 0)
    cpu_id: CpuId,
}

impl Cpu {
    pub fn id(&self) -> CpuId {
        self.cpu_id
    }
}

pub struct IrqController;

impl IrqController {
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
        let rflags: Flags;
        unsafe {
            asm!("pushfq; pop $0" : "=r"(rflags) : : "memory" : "intel", "volatile");
        }
        rflags.contains(Flags::IF)
    }
}

pub unsafe fn init(cpu_id: u32) {
    let cpu = Box::new(Cpu {
        cpu_id,
    });

    let mut cpu_local = Box::new(Local::new(Box::leak(cpu)));

    cpu_local.direct = (&*cpu_local).into();

    Msr::new(0xC0000101)
        .write(Box::into_raw(cpu_local) as u64);
}

// /// Global system data
// pub struct Global {
//     /// List of all locals that are currently online.
//     pub locals: RwLock<Vec<Local>>,
// }

// impl Global {
//     fn new() -> Global {
//         Global {
//             locals: RwLock::new(Vec::new()),
//         }
//     }
// }

/// Each cpu contains this in the gs register.
pub struct Local {
    direct: NonNull<Local>,
    /// Reference to the current `Cpu`.
    pub cpu: &'static mut Cpu,
    /// The scheduler associated with this cpu.
    pub scheduler: Scheduler,
    current_thread: NonNull<Thread>,
}

impl Local {
    fn new(cpu: &'static mut Cpu) -> Local {
        let idle_thread = Box::new(Thread::new(4096, Box::new(|| {
            loop {
                unsafe { ::arch::interrupt::halt(); }
            }
        })).unwrap());

        let mut kernel_thread = Box::new(Thread::new(0, Box::new(|| {}))
            .unwrap());
            
        kernel_thread.state = State::Suspended;

        let kernel_thread_nonnull = Box::into_raw_non_null(kernel_thread);

        let scheduler = Scheduler::new(
            kernel_thread_nonnull.as_ptr(),
            Box::into_raw(idle_thread)
        );

        Local {
            direct: NonNull::dangling(),
            cpu,
            scheduler,
            current_thread: kernel_thread_nonnull,
        }
    }

    pub fn current() -> &'static mut Local {
        unsafe {
            &mut *(read_gs_offset64!(0x0) as *mut Local)
        }
    }

    pub fn current_thread() -> NonNull<Thread> {
        Self::current().current_thread
    }

    pub fn set_current_thread(ptr: NonNull<Thread>) {
        Self::current().current_thread = ptr;
    }
}
