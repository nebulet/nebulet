use wasm::compile_module;
use alloc::Vec;

static WASM_TESTS: [&'static [u8]; 6] = [
    include_bytes!("wasmtests/arith.wasm"),
    include_bytes!("wasmtests/call.wasm"),
    include_bytes!("wasmtests/fibonacci.wasm"),
    include_bytes!("wasmtests/globals.wasm"),
    include_bytes!("wasmtests/memory.wasm"),
    include_bytes!("wasmtests/exit.wasm"),
];

pub fn wasm_test() -> Result<(), ()> {
    let mut codes = Vec::new();
    for (i, wasm) in WASM_TESTS.iter().enumerate() {
        println!("Compiling wasm test #{}", i);
        match compile_module(wasm) {
            Ok(code) => codes.push((i, code)),
            Err(err) => println!("Wasm test #{} failed to compile: {:?}", i, err),
        }
    }

    for (i, code) in codes.iter_mut() {
        println!("Executing wasm test #{}", i);
        code.execute();
    }

    Ok(())
}

