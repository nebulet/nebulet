
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        #[cfg(feature = "vga")]
        $crate::arch::printer::_print(format_args!($($arg)*));
        #[cfg(feature = "serial")]
        $crate::arch::devices::serial::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub macro interrupt($name:ident, $func:block) {
    use x86_64::structures::idt::ExceptionStackFrame;
    #[allow(unused_unsafe)]
    pub extern "x86-interrupt" fn $name (_: &mut ExceptionStackFrame) {
        unsafe {
            $func
        }
    }
}

pub macro interrupt_stack($name:ident, $stack:ident, $func:block) {
    use x86_64::structures::idt::ExceptionStackFrame;
    #[allow(unused_unsafe)]
    pub extern "x86-interrupt" fn $name ($stack: &mut ExceptionStackFrame) {
        unsafe {
            $func
        }
    }
}

pub macro interrupt_stack_err($name:ident, $stack:ident, $error:ident, $func:block) {
    use x86_64::structures::idt::ExceptionStackFrame;
    #[allow(unused_unsafe)]
    pub extern "x86-interrupt" fn $name ($stack: &mut ExceptionStackFrame, $error: u64) {
        unsafe {
            $func
        }
    }
}

pub macro interrupt_stack_page($name:ident, $stack:ident, $error:ident, $func:block) {
    use x86_64::structures::idt::{ExceptionStackFrame, PageFaultErrorCode};
    #[allow(unused_unsafe)]
    pub extern "x86-interrupt" fn $name ($stack: &mut ExceptionStackFrame, $error: PageFaultErrorCode) {
        unsafe {
            $func
        }
    }
}

macro_rules! likely {
    ($e:expr) => {
        unsafe {
            ::core::intrinsics::likely($e)
        }
    };
}

macro_rules! unlikely {
    ($e:expr) => {
        unsafe {
            ::core::intrinsics::unlikely($e)
        }
    };
}

macro_rules! offset_of {
    ($ty:ty:$field:ident) => {
        #[allow(unused_unsafe)]
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    }
}
