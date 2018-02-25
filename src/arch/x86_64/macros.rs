#[cfg(feature = "vga")]
use super::printer;

pub macro print($($arg:tt)*) {
    #[cfg(feature = "vga")]
    printer::_print(format_args!($($arg)*));
}

pub macro println {
    () => (print!("\n")),
    ($fmt:expr) => (print!(concat!($fmt, "\n"))),
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*))
}

pub macro interrupt($name:ident, $func:block) {
    use x86_64::structures::idt::ExceptionStackFrame;
    pub extern "x86-interrupt" fn $name (_: &mut ExceptionStackFrame) {
        unsafe {
            $func
        }
    }
}

pub macro interrupt_stack($name:ident, $stack:ident, $func:block) {
    use x86_64::structures::idt::ExceptionStackFrame;
    pub extern "x86-interrupt" fn $name ($stack: &mut ExceptionStackFrame) {
        unsafe {
            $func
        }
    }
}