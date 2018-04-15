use os_bootinfo::BootInfo;
use arch::memory;
use allocator;
use arch::{idt, interrupt, devices, paging, cpu};

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

    cpu::init();
    
    memory::init(boot_info);

    // Initialize paging
    let mut page_mapper = paging::init();

    // Initialize dynamic memory allocation
    allocator::init(&mut page_mapper);

    // Initialize the IDT
    idt::init();

    // Initialize essential devices
    devices::init();

    // Initialize non-essential devices
    devices::init_noncore();

    interrupt::enable();

    ::kmain(1);
}