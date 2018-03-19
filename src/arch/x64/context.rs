use core::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
use core::mem;

#[derive(Debug)]
pub struct Context {
    pub rflags: usize,
    pub rbx: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
    pub rbp: usize,
    pub rsp: usize,
}

impl Context {
    pub fn new() -> Self {
        Self {
            rflags: 0,
            rbx: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rbp: 0, rsp: 0,
        }
    }

    pub unsafe fn push_stack(&mut self, val: usize) {
        self.rsp -= mem::size_of::<usize>();
        *(self.rsp as *mut usize) = val;
    }

    /// Switch to a new context
    #[naked]
    #[inline(never)]
    pub unsafe extern "C" fn switch_to(&mut self, next: &Context) {
        asm!("pushfq; pop $0" : "=r"(self.rflags) : : "memory" : "intel", "volatile");
        asm!("push $0; popfq" : : "r"(next.rflags) : "memory" : "intel", "volatile");

        asm!("mov $0, rbx" : "=r"(self.rbx) : : "memory" : "intel", "volatile");
        asm!("mov $0, r12" : "=r"(self.r12) : : "memory" : "intel", "volatile");
        asm!("mov $0, r13" : "=r"(self.r13) : : "memory" : "intel", "volatile");
        asm!("mov $0, r14" : "=r"(self.r14) : : "memory" : "intel", "volatile");
        asm!("mov $0, r15" : "=r"(self.r15) : : "memory" : "intel", "volatile");
        asm!("mov $0, rsp" : "=r"(self.rsp) : : "memory" : "intel", "volatile");
        asm!("mov $0, rbp" : "=r"(self.rbp) : : "memory" : "intel", "volatile");

        asm!("mov rbx, $0" : : "r"(next.rbx) : "memory" : "intel", "volatile");
        asm!("mov r12, $0" : : "r"(next.r12) : "memory" : "intel", "volatile");
        asm!("mov r13, $0" : : "r"(next.r13) : "memory" : "intel", "volatile");
        asm!("mov r14, $0" : : "r"(next.r14) : "memory" : "intel", "volatile");
        asm!("mov r15, $0" : : "r"(next.r15) : "memory" : "intel", "volatile");
        asm!("mov rsp, $0" : : "r"(next.rsp) : "memory" : "intel", "volatile");
        asm!("mov rbp, $0" : : "r"(next.rbp) : "memory" : "intel", "volatile");
    }
}

