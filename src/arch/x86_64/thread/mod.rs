use task::{self, Thread, ThreadState, ThreadFlags};
use mp::PerCpu;
use macros::println;
use asm;

use core::{mem, ptr};

global_asm!(include_str!("asm.s"));
extern "C" {
    fn x86_64_context_switch(old_sp: &mut usize, new_sp: usize);
}

#[derive(Debug, PartialEq, Eq)]
pub struct Context {
    pub sp: usize,
    pub fs_base: usize,
    pub gs_base: usize,
}

impl Context {
    pub fn new() -> Context {
        Context {
            sp: 0,
            fs_base: 0,
            gs_base: 0,
        }
    }
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

pub unsafe fn thread_initialize(thread: &mut Thread) {
    let stack_top = {
        let mut stack_top = thread.stack.as_ptr() as usize + thread.stack.len();
        // round down to align at 16 bytes
        stack_top = stack_top & !15;
        // make sure we start the frame 8 byte unaligned
        stack_top -= 8;

        stack_top
    };

    let frame_ptr = (stack_top as *mut ContextSwitchFrame).offset(-1);

    let frame = ContextSwitchFrame {
        r15: 0, r14: 0, r13: 0, r12: 0,
        rbp: 0,
        rbx: 0,
        rflags: 0x3002,
        rip: initial_thread_entry as usize,
    };

    ptr::write(frame_ptr, frame);

    let debug_frame = &*frame_ptr;

    thread.arch.sp = frame_ptr as usize;
}

pub unsafe fn context_switch(old_thread: &mut Thread, new_thread: &mut Thread) {
    x86_64_context_switch(&mut old_thread.arch.sp, new_thread.arch.sp);
}

extern fn initial_thread_entry() -> ! {
    let thread = unsafe { get_current_thread() };
    
    let ret = (thread.entry)(thread.arg);

    println!("Thread return: {}", ret);

    thread_exit(ret);

    unreachable!();
}

fn thread_exit(retcode: i32) {
    let mut current_thread = get_current_thread();
    
    debug_assert!(current_thread.state == ThreadState::Running);

    current_thread.state = ThreadState::Dead;
    current_thread.retcode = retcode;

    if current_thread.flags.contains(ThreadFlags::DETACHED) {
        drop(&current_thread.stack);
    }

    task::resched();

    println!("Whoops, somehow fell through thread::exit");
}

#[inline]
pub fn get_current_thread() -> &'static mut Thread {
    unsafe {
        &mut *(asm::read_gs_offset64!(PerCpu::CURRENT_THREAD_OFFSET) as *mut Thread)
    }
}

#[inline]
pub fn set_current_thread(thread: &mut Thread) {
    unsafe {
        asm::write_gs_offset64!(PerCpu::CURRENT_THREAD_OFFSET, thread as *mut Thread as u64);
    }
}