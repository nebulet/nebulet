use nebulet_derive::nebulet_abi;
use wasm::{VmCtx, UserData};
use alloc::string::String;
use x86_64::instructions::port::Port;

#[nebulet_abi]
pub fn print(buffer_offset: u32, buffer_size: u32, user_data: &UserData) {
    let memory = user_data.instance.memories[0].read();
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
