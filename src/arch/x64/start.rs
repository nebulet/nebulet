use os_bootinfo::BootInfo;
use arch::memory;
use arch::{idt, interrupt, devices, paging, cpu};
use dpc;

/// Test of zero values in BSS.
static BSS_TEST_ZERO: usize = 0x0;
/// Test of non-zero values in data.
static DATA_TEST_NONZERO: usize = 0xFFFF_FFFF_FFFF_FFFF;

/// This function is extremely unsafe
/// Thus, it is marked unsafe
#[no_mangle]
pub unsafe fn _start(boot_info_ptr: *mut BootInfo) -> ! {
    let boot_info = &mut*boot_info_ptr;

    boot_info.check_version().unwrap();

    // .bss section should be zeroed
    {
        assert_eq!(BSS_TEST_ZERO, 0x0);
        assert_eq!(DATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFF);
    }

    interrupt::disable();
    
    memory::init(boot_info);

    // Initialize paging
    paging::init();

    // Initialize the IDT
    idt::init();
    
    // Initialize the cpu and cpu local structures
    cpu::init(0);

    // Initialize essential devices
    devices::init();

    // Initialize non-essential devices
    devices::init_noncore();

    // Initialize deferred procedure calls
    dpc::init();

    ::kmain();
}
