#![no_main]

extern crate userspace;

#[no_mangle]
pub fn main() {
    let b = Box::new(42);

    userspace::print(&format!("boxed in wasm: {:?}", b));
}
