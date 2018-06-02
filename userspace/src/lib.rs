#![no_std]
#![feature(lang_items)]
#![feature(start)]
#![feature(wasm_import_module)]

#[lang = "panic_fmt"]
fn panic_fmt() -> ! {
    loop {}
}

#[lang = "oom"]
fn oom() -> ! {
    loop {}
}

// This does nothing but satisfy the rust compiler
// (well actually it exports a `main` function that calls this, but it doesn't
// have the type we want it to have).
#[start]
pub fn rust_start_not_called(_argc: isize, _argv: *const *const u8) -> isize {
    0xbadbad
}

pub mod abi {
    #[wasm_import_module = "abi"]
    extern {
        pub fn print(ptr: *const u8, len: usize);
    }
}

pub fn print(x: &str) {
    unsafe {
        abi::print(x.as_ptr(), x.len());
    }
}
