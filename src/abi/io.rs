use object::ProcessRef;
use nebulet_derive::nebulet_abi;
use alloc::String;

#[nebulet_abi]
pub fn print(buffer_offset: u32, buffer_size: u32, process: &ProcessRef) {
    let instance = process.instance().read();
    let memory = &instance.memories[0];

    if let Some(buf) = memory.carve_slice(buffer_offset, buffer_size) {
        let s = String::from_utf8_lossy(buf);
        println!("{}", s);
    }
    else {
        println!("\nPrinting invalid buffer!")
    }
}