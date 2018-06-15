use object::ProcessRef;
use nebulet_derive::nebulet_abi;
use wasm::instance::VmCtx;
use alloc::String;
use x86_64::instructions::port::Port;

#[nebulet_abi]
pub fn print(buffer_offset: u32, buffer_size: u32, process: &ProcessRef) {
    let instance = process.instance().read();
    let memory = &instance.memories[0];
    if let Some(buf) = memory.carve_slice(buffer_offset, buffer_size) {
        let s = String::from_utf8_lossy(buf);
        print!("{}", s);
    }
    else {
        println!("\nPrinting invalid buffer!")
    }
}

pub unsafe fn read_port_u8(port: u32, _: &VmCtx) -> u32 {
    Port::<u8>::new(port as u16)
        .read() as u32
}

pub unsafe fn write_port_u8(port: u32, val: u32, _: &VmCtx) {
    Port::<u8>::new(port as u16)
        .write(val as u8);
}
