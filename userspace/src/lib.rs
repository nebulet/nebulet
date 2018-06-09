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
        pub fn process_create(code_handle: u32, chan_handle: u32) -> u64;
        pub fn process_start(process_handle: u32) -> u64;
        
        pub fn channel_create(handle0: &mut u32, handle1: &mut u32) -> u64;
        pub fn channel_write(handle: u32, ptr: *const u8, len: usize) -> u64;
        pub fn channel_read(handle: u32, ptr: *mut u8, len: usize, msg_len_out: &mut usize) -> u64;

        pub fn physical_map(phys_addr: u64, page_count: usize) -> u64;
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

pub fn process_create(handle: u32, chan_handle: u32) -> nabi::Result<u32> {
    let ret = unsafe {
        abi::process_create(handle, chan_handle)
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
    let (mut handle_tx, mut handle_rx) = (0, 0);
    let ret = unsafe {
        abi::channel_create(&mut handle_tx, &mut handle_rx)
    };

    nabi::Error::demux(ret)
        .map(|_| (handle_tx, handle_rx))
}

pub fn channel_write(handle: u32, data: &[u8]) -> nabi::Result<()> {
    let ret = unsafe {
        abi::channel_write(handle, data.as_ptr(), data.len())
    };

    nabi::Error::demux(ret).map(|_| ())
}

pub fn channel_read(handle: u32, buffer: &mut [u8]) -> (usize, nabi::Result<()>) {
    let mut msg_size_out = 0;
    let ret = unsafe {
        abi::channel_read(handle, buffer.as_mut_ptr(), buffer.len(), &mut msg_size_out)
    };

    (msg_size_out, nabi::Error::demux(ret).map(|_| ()))
}

pub fn physical_map<T: Sized>(phys_addr: u64) -> nabi::Result<&'static mut T> {
    use std::mem;

    let page_count = {
        let rem = mem::size_of::<T>() % (1 << 16);
        mem::size_of::<T>() + (1 << 16) - rem
    };

    let ret = unsafe {
        abi::physical_map(phys_addr, page_count)
    };

    nabi::Error::demux(ret)
        .map(|offset| unsafe { mem::transmute::<_,  &'static mut T>(offset) })
}
