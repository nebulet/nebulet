use x86_64::registers::model_specific::Msr;
use x86_64::registers::rflags::RFlags;
use core::ptr::NonNull;

use arch::interrupt;
use arch::asm::read_gs_offset64;

use task::scheduler::Scheduler;
use object::thread::{Thread, State};

use alloc::boxed::Box;
use event::{Event, EventVariant};
use sync::mpsc::{Mpsc, IntrusiveMpsc};

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
    /// Reference to the local `Cpu`.
    _cpu: *const Cpu,
    /// The scheduler associated with this cpu.
    scheduler: Scheduler,
    /// Pointer to current thread.
    pub current_thread: *mut Thread,
    /// The dpc instance local to this cpu
    dpc: Dpc,
}

impl Local {
    unsafe fn new(cpu: *const Cpu) -> Local {
        let idle_thread = Thread::new(4096, || {
            loop {
                ::arch::interrupt::halt();
            }
        }).unwrap();

        let kernel_thread = Thread::new(4096, || {}).unwrap();

        idle_thread.set_state(State::Ready);
        kernel_thread.set_state(State::Dead);

        let scheduler = Scheduler::new(Box::into_raw(idle_thread));

        let (dpc_thread, dpc) = Dpc::new();

        dpc_thread.set_state(State::Ready);
        scheduler.schedule_thread(Box::into_raw(dpc_thread));

        Local {
            direct: NonNull::dangling(),
            _cpu: cpu,
            scheduler,
            current_thread: Box::into_raw(kernel_thread),
            dpc,
        }
    }

    pub fn current() -> &'static mut Local {
        unsafe {
            &mut *(read_gs_offset64!(0x0) as *mut Local)
        }
    }

    #[inline]
    pub fn current_thread() -> *mut Thread {
        unsafe {
            read_gs_offset64!(offset_of!(Local, current_thread)) as *mut Thread
        }
    }

    #[inline]
    pub fn set_current_thread(thread: *mut Thread) {
        unsafe {
            asm!("mov $0, %gs:0x28" : : "r"(thread) : "memory" : "volatile");
        }
    }

    pub fn schedule_thread(thread: *mut Thread) {
        Self::current().scheduler.schedule_thread(thread);
    }

    pub unsafe fn context_switch() {
        Self::current().scheduler.switch();
    }
}

pub struct Dpc {
    runqueue: Mpsc<(usize, fn(usize))>,
    thread_cleanup_queue: IntrusiveMpsc<Thread>,
    event: Event,
}

impl Dpc {
    fn new() -> (Box<Thread>, Dpc) {
        let dpc_thread = Thread::new(4096 * 4, || {
            let local = Local::current();
            loop {
                local.dpc.event.wait();
                while let Some((arg, f)) = unsafe { local.dpc.runqueue.pop() } {
                    f(arg);
                }
                while let Some(thread) = unsafe { local.dpc.thread_cleanup_queue.pop() } {
                    let boxed_thread = unsafe { Box::from_raw(thread) };
                    debug_assert!(boxed_thread.state() == State::Dead);
                }
            }
        }).expect("Enable to create dpc thread");

        (dpc_thread, Dpc {
            runqueue: Mpsc::new(),
            thread_cleanup_queue: IntrusiveMpsc::new(),
            event: Event::new(EventVariant::AutoUnsignal),
        })
    }

    pub fn queue(arg: usize, f: fn(usize)) {
        let local = Local::current();
        local.dpc.runqueue.push((arg, f));
        local.dpc.event.signal(false);
    }

    pub fn cleanup_thread(thread: *mut Thread) {
        let local = Local::current();
        unsafe {
            local.dpc.thread_cleanup_queue.push(thread);
        }
        local.dpc.event.signal(false);
    }
}
