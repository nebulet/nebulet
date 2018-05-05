use core::{mem, ptr};

global_asm!("
.global x86_64_context_switch
.intel_syntax noprefix
x86_64_context_switch:
    pushfq
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15

    mov [rdi], rsp
    mov rsp, rsi

    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
    popfq

    ret
");

extern "C" {
    fn x86_64_context_switch(oldsp: &mut usize, newsp: usize);
}

#[repr(C)]
pub struct ContextSwitchFrame {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    rbp: usize,
    rbx: usize,
    rflags: usize,
    rip: usize,
}

#[derive(Debug, Clone)]
pub struct Context {
    rsp: usize,
}

impl Context {
    pub fn from_rsp(rsp: usize) -> Context {
        Context {
            rsp,
        }
    }
    /// The returned tuple on this is ugly, but I couldn't think
    /// of a better way of doing it.
    pub fn init(stack_top: *mut u8, entry: extern fn(), stack_item: usize) -> Self {
        fn round_down(addr: usize, align: usize) -> usize {
            addr & !(align - 1)
        }

        let faux_frame = ContextSwitchFrame {
            r15: 0, r14: 0, r13: 0, r12: 0,
            rbp: 0,
            rbx: 0,
            rflags: 0x0200, // IF = 1, NT = 0, IOPL = 0
            rip: entry as _,
        };

        let rounded = round_down(stack_top as usize, 16) - 8;

        unsafe {
            *(rounded as *mut usize) = stack_item;
        }

        let adjusted_stack_top = rounded - mem::size_of::<ContextSwitchFrame>();

        unsafe {
            ptr::write(adjusted_stack_top as *mut _, faux_frame);
        }

        Context {
            rsp: adjusted_stack_top,
        }
    }

    pub unsafe fn push_stack(&mut self, val: usize) {
        self.rsp -= mem::size_of::<usize>();
        *(self.rsp as *mut usize) = val;
    }

    /// Switch to a new context
    pub unsafe extern "C" fn switch_to(&mut self, next: &Context) {
        x86_64_context_switch(&mut self.rsp, next.rsp);
    }
}
