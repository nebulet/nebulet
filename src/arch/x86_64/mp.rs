use task::Thread;

use x86_64::registers::model_specific::Msr;
use x86_64::registers::msr::IA32_GS_BASE;

const SMP_MAX_CPUS: usize = 16;

static mut CPU0: PerCpu = PerCpu {
    direct: 0 as *mut PerCpu,
    current_thread: 0 as *mut Thread,
    in_irq: 0,
    cpu_num: 0,
};

#[repr(C, packed)]
pub struct PerCpu {
    // Direct pointer to self
    direct: *mut PerCpu,

    // The current thread
    current_thread: *mut Thread,

    // currently in irq
    in_irq: u32,

    cpu_num: u32,
}

impl PerCpu {
    pub const DIRECT_OFFSET: usize          = 0x0;
    pub const CURRENT_THREAD_OFFSET: usize  = 0x8;
    pub const IN_IRQ_OFFSET: usize          = 0x10;
    pub const CPU_NUM_OFFSET: usize         = 0x14;
}

pub unsafe fn init() {
    CPU0.direct = &mut CPU0 as *mut PerCpu;

    let mut msr = Msr(IA32_GS_BASE);

    msr.write(CPU0.direct as u64);
}