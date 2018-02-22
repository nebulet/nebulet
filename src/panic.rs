
use arch::macros::println;
use core::fmt;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(msg: fmt::Arguments, _file: &'static str, _line: u32, _col: u32) -> ! {
    // println!("panic {} in {} at line {}:{}", msg, file, line, col);
    println!("panic: {}", msg);
    loop {}
}