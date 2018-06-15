use x86_64::registers::model_specific::Msr;
use x86_64::registers::rflags::RFlags;
use core::ptr::NonNull;

use arch::interrupt;
use arch::asm::read_gs_offset64;

use task::{State, scheduler::Scheduler};
use common::mpsc::Sender;

use alloc::boxed::Box;
use nil::Ref;
use object::ThreadRef;

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
        let rflags: RFlags;
        unsafe {
            asm!("pushfq; pop $0" : "=r"(rflags) : : "memory" : "intel", "volatile");
        }
        rflags.contains(RFlags::INTERRUPT_FLAG)
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
    scheduler: Scheduler,
    /// Pointer to current thread.
    current_thread: Ref<ThreadRef>,
    /// Sender for threads
    thread_tx: Sender<Ref<ThreadRef>>,
}

impl Local {
    fn new(cpu: &'static mut Cpu) -> Local {
        let idle_thread = ThreadRef::new(unsafe { Ref::dangling() }, 4096, || {
            loop {
                unsafe { ::arch::interrupt::halt(); }
            }
        }).unwrap();

        let kernel_thread = ThreadRef::new(unsafe { Ref::dangling() }, 4096, || {}).unwrap();

        idle_thread.set_state(State::Ready);
        kernel_thread.set_state(State::Ready);

        let scheduler = Scheduler::new(idle_thread);

        let thread_tx = scheduler.thread_sender();

        Local {
            direct: NonNull::dangling(),
            cpu,
            scheduler,
            current_thread: kernel_thread,
            thread_tx,
        }
    }

    pub fn current() -> &'static mut Local {
        unsafe {
            &mut *(read_gs_offset64!(0x0) as *mut Local)
        }
    }

    pub fn current_thread() -> Ref<ThreadRef> {
        Self::current().current_thread.clone()
    }

    pub fn set_current_thread(thread: Ref<ThreadRef>) {
        Self::current().current_thread = thread;
    }

    pub fn schedule_thread(thread: Ref<ThreadRef>) {
        Self::current().thread_tx.send(thread);
    }

    pub unsafe fn context_switch() {
        Self::current().scheduler.switch();
    }
}
