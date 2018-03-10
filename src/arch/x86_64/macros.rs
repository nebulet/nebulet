#[cfg(feature = "vga")]
use super::printer;
#[cfg(feature = "serial")]
use devices::serial;
use interrupt;

pub macro print($($arg:tt)*) {
    #[cfg(feature = "vga")]
    printer::_print(format_args!($($arg)*));
    #[cfg(feature = "serial")]
    serial::_print(format_args!($($arg)*));
}

pub macro println {
    () => (print!("\n")),
    ($fmt:expr) => (print!(concat!($fmt, "\n"))),
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*))
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
        // for now, always dump the stack
        println!("{:?}", $stack);
    }
}

pub macro interrupt_stack_err($name:ident, $stack:ident, $error:ident, $func:block) {
    use x86_64::structures::idt::ExceptionStackFrame;
    #[allow(unused_unsafe)]
    pub extern "x86-interrupt" fn $name ($stack: &mut ExceptionStackFrame, $error: u64) {
        unsafe {
            $func
        }
        // for now, always dump the stack
        println!("{:?}", $stack);
        println!("Error: {}", $error);
    }
}

pub macro interrupt_stack_page($name:ident, $stack:ident, $error:ident, $func:block) {
    use x86_64::structures::idt::{ExceptionStackFrame, PageFaultErrorCode};
    #[allow(unused_unsafe)]
    pub extern "x86-interrupt" fn $name ($stack: &mut ExceptionStackFrame, $error: PageFaultErrorCode) {
        unsafe {
            $func
        }
        // for now, always dump the stack
        println!("{:?}", $stack);
        println!("PageError: {:?}", $error);
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