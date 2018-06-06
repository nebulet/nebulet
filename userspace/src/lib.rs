#![feature(
    wasm_import_module,
    global_allocator,
)]

extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
