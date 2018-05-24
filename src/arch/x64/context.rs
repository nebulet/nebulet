use core::mem;
use x86_64::registers::flags::Flags;

extern "C" {
    fn x86_64_context_switch(prev: *mut ThreadContext, next: *const ThreadContext);
}

#[derive(Debug)]
#[repr(C)]
pub struct ThreadContext {
    rflags: usize,
    rbx: usize,
    r12: usize,
    r13: usize,
    r14: usize,
    r15: usize,
    rbp: usize,
    rsp: usize,
}

impl ThreadContext {
    pub fn new(stack_top: *mut u8, entry: extern fn()) -> ThreadContext {
        let mut ctx = ThreadContext {
            rflags: Flags::IF.bits(),
            rbx: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rbp: stack_top as _,
            rsp: stack_top as usize,
        };

        unsafe {
            ctx.push_stack(entry as _);
        }

        ctx
    }

    /// Push an item onto the `ThreadContext`'s stack.
    pub unsafe fn push_stack(&mut self, item: usize) {
        self.rsp -= mem::size_of::<usize>();
        *(self.rsp as *mut usize) = item;
    }

    #[inline]
    pub unsafe fn swap(&mut self, next: &ThreadContext) {
        x86_64_context_switch(self as *mut _, next as *const _);
    }
}
