use core::fmt;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(msg: fmt::Arguments, file: &'static str, line: u32, col: u32) -> ! {
    println!("panic: {} in {} at line {}:{}", msg, file, line, col);
    loop {}
}