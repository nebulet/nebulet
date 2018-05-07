use core::mem;
use x86_64::registers::flags::Flags;

global_asm!("
.global x86_64_context_switch
.intel_syntax noprefix

# ThreadContext {
#   0x0: flags
#   0x8: rbx
#   0x10: r12
#   0x18: r13
#   0x20: r14
#   0x28: r15
#   0x30: rbp
#   0x38: rsp
# }
#
# rdi <- reference to previous `ThreadContext`
# rsi <- reference to next `ThreadContext`
x86_64_context_switch:
    # Save the previous context
    pushfq
    pop qword ptr [rdi] # save rflags into prev.flags

    mov [rdi+0x8], rbx  # save rbx
    mov [rdi+0x10], r12 # save r12
    mov [rdi+0x18], r13 # save r13
    mov [rdi+0x20], r14 # save r14
    mov [rdi+0x28], r15 # save r15
    mov [rdi+0x30], rbp # save rbp
    
    # Swap the stack pointers
    mov [rdi+0x38], rsp # save rsp
    mov rsp, [rsi+0x38] # set rsp

    # Switch to the next context
    mov rbp, [rsi+0x30] # set rbp
    mov r15, [rsi+0x28] # set r15
    mov r14, [rsi+0x20] # set r14
    mov r13, [rsi+0x18] # set r13
    mov r12, [rsi+0x10] # set r12
    mov rbx, [rsi+0x8]  # set rbx

    push [rsi] # set rflags
    popfq
    
    # leap of faith
    ret
");

extern "C" {
    fn x86_64_context_switch(prev: *mut ThreadContext, next: *const ThreadContext);
}

#[derive(Debug)]
#[repr(C)]
pub struct ThreadContext {
    rflags: usize,
    pub rbx: usize,
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