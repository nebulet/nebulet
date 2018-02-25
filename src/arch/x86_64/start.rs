use os_bootinfo::BootInfo;
use idt;
use interrupt;
use devices;
use memory;
use macros::println;

/// Test of zero values in BSS.
static BSS_TEST_ZERO: usize = 0x0;
/// Test of non-zero values in data.
static DATA_TEST_NONZERO: usize = 0xFFFF_FFFF_FFFF_FFFF;

#[inline]
fn initial_asserts() {
    // .bss section should be zeroed
    assert_eq!(BSS_TEST_ZERO, 0x0);
    assert_eq!(DATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFF);
}

/// This function is extremely unsafe
#[no_mangle]
pub unsafe fn _start(boot_info_ptr: *mut BootInfo) -> ! {
    let boot_info = unsafe {
        &mut*boot_info_ptr
    };

    // make sure the starting environment is OK
    initial_asserts();

    memory::init(boot_info);

    // Initialize the IDT
    idt::init();

    // Initialize essential devices
    devices::init();

    // Initialize non-essential devices
    devices::init_noncore();

    println!("OK");

    asm!("int3");

    loop {
        unsafe { interrupt::halt();}
    }
}