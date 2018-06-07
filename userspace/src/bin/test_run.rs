#![no_main]

#[macro_use]
extern crate userspace;

fn run_proc(wasm: &[u8]) {
    let compile_ret = userspace::compile_wasm(wasm).unwrap();
    let create_ret = userspace::process_create(compile_ret).unwrap();
    userspace::process_start(create_ret).unwrap();
}

#[no_mangle]
pub fn main() {
    println!("Executing packaged wasm.");

    let wasm0 = include_bytes!("../../target/wasm32-unknown-unknown/release/hello.wasm");
    let wasm1 = include_bytes!("../../target/wasm32-unknown-unknown/release/chan.wasm");

    run_proc(wasm1);
    run_proc(wasm0);

    loop {}
}
