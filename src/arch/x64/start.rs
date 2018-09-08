use arch::memory;
use arch::{cpu, devices, idt, interrupt, paging, pci};
use bootloader::bootinfo::BootInfo;

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

        memory::init(boot_info, 4 * 1024 * 1024 /* 4 MB */);

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

    find_pci_devices();

    ::kmain(&boot_info.package);
}

entry_point!(arch_start);

fn find_pci_devices() {
    for bus in 0..=255 {
        let bus = pci::PciBus::new(bus);
        bus.scan(|device| {
            if device.vendor == 0x8086 && device.device == 0x100e {
                println!("device bar: {:#x}", device.base_address());
                println!("header: {:#x}", device.header_type());
                println!("{:#x?}", device);
            }
        });
    }
}
