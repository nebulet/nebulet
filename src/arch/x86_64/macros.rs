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