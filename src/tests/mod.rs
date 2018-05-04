pub mod wasm;

pub fn test_all() {
    wasm::wasm_test().unwrap();

    println!("All tests passed.");
}