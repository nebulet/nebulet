#![feature(
    wasm_import_module,
    global_allocator,
)]

extern crate wee_alloc;
extern crate nabi;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::print(&format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

mod abi {
    #[wasm_import_module = "abi"]
    extern {
        pub fn print(ptr: *const u8, len: usize);
        pub fn wasm_compile(ptr: *const u8, len: usize) -> u64;
        pub fn process_create(code_handle: u32) -> u64;
        pub fn process_start(process_handle: u32) -> u64;
        
        pub fn channel_create(handle0: &mut u32, handle1: &mut u32) -> u64;
    }
}

pub fn print(x: &str) {
    unsafe {
        abi::print(x.as_ptr(), x.len());
    }
}

pub fn compile_wasm(wasm: &[u8]) -> nabi::Result<u32> {
    let ret = unsafe {
        abi::wasm_compile(wasm.as_ptr(), wasm.len())
    };

    nabi::Error::demux(ret)
}

pub fn process_create(handle: u32) -> nabi::Result<u32> {
    let ret = unsafe {
        abi::process_create(handle)
    };

    nabi::Error::demux(ret)
}

pub fn process_start(handle: u32) -> nabi::Result<u32> {
    let ret = unsafe {
        abi::process_start(handle)
    };

    nabi::Error::demux(ret)
}

pub fn channel_create() -> nabi::Result<(u32, u32)> {
    let (mut handle0, mut handle1) = (0, 0);
    let ret = unsafe {
        abi::channel_create(&mut handle0, &mut handle1)
    };

    nabi::Error::demux(ret)
        .map(|_| (handle0, handle1))
}
