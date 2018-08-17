use bootloader::bootinfo::BootInfo;
use arch::memory;
use arch::{idt, interrupt, devices, paging, cpu};

/// Test of zero values in BSS.
static BSS_TEST_ZERO: usize = 0x0;
/// Test of non-zero values in data.
static DATA_TEST_NONZERO: usize = 0xFFFF_FFFF_FFFF_FFFF;

fn arch_start(boot_info: &'static BootInfo) -> ! {
    // .bss section should be zeroed
    {
        assert_eq!(BSS_TEST_ZERO, 0x0);
        assert_eq!(DATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFF);
    }

    unsafe {
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
    }

    ::kmain(&boot_info.package);
}

entry_point!(arch_start);

// /// This function is extremely unsafe
// /// Thus, it is marked unsafe
// #[no_mangle]
// pub unsafe fn _start(boot_info_ptr: *mut BootInfo) -> ! {
//     let boot_info = &mut*boot_info_ptr;

//     boot_info.check_version().unwrap();

//     let package = mem::replace(&mut boot_info.package, mem::zeroed());

//     // .bss section should be zeroed
//     {
//         assert_eq!(BSS_TEST_ZERO, 0x0);
//         assert_eq!(DATA_TEST_NONZERO, 0xFFFF_FFFF_FFFF_FFFF);
//     }

//     interrupt::disable();
    
//     memory::init(boot_info);

//     // Initialize paging
//     paging::init();

//     // Initialize the IDT
//     idt::init();
    
//     // Initialize the cpu and cpu local structures
//     cpu::init(0);

//     // Initialize essential devices
//     devices::init();

//     // Initialize non-essential devices
//     devices::init_noncore();

//     ::kmain(&package);
// }
