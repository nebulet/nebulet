use os_bootinfo::BootInfo;
use super::interrupt;
use memory;
use super::macros::println;
use super::printer::PRINTER;

#[no_mangle]
pub fn _start(boot_info_ptr: *mut BootInfo) -> ! {
    let boot_info = unsafe {
        &mut*boot_info_ptr
    };

    // println!("Memory Map: {:?}", boot_info.memory_map);

    memory::init(boot_info);

    // interrupt::init();
    
    // unsafe { interrupt::enable_and_nop() };

    println!("A-OK");

    loop {
        // unsafe { interrupt::halt();}
    }
}