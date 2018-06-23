use core::mem;
use x86_64::registers::rflags::RFlags;

extern {
    fn x86_64_context_switch(prev: *mut ThreadContext, next: *const ThreadContext);
}

#[derive(Debug)]
#[repr(C)]
pub struct ThreadContext {
    rflags: u64,
    rbx: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rbp: u64,
    rsp: u64,
}

impl ThreadContext {
    pub fn new(stack_top: *mut u8, entry: extern fn()) -> ThreadContext {
        let mut ctx = ThreadContext {
            rflags: RFlags::INTERRUPT_FLAG.bits(),
            rbx: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rbp: stack_top as _,
            rsp: stack_top as _,
        };

        unsafe {
            ctx.push_stack(entry as _);
        }

        ctx
    }

    /// Push an item onto the `ThreadContext`'s stack.
    pub unsafe fn push_stack(&mut self, item: usize) {
        self.rsp -= mem::size_of::<usize>() as u64;
        *(self.rsp as *mut usize) = item;
    }

    #[inline]
    pub unsafe fn swap(&mut self, next: &ThreadContext) {
        x86_64_context_switch(self as *mut _, next as *const _);
    }
}
