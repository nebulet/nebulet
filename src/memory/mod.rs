//! Blanket module for memory things
//! Allocator, paging (although there isn't much), etc

use os_bootinfo::BootInfo;
use x86_64::structures::paging::{PageTable, PageTableFlags};
use arch::macros::println;
// use spin::Mutex;

use core::sync::atomic::{AtomicPtr, Ordering};

// Defaults to null
static P4_TABLE: AtomicPtr<PageTable> = AtomicPtr::new(::core::ptr::null_mut());

// static FRAME_ALLOCATOR: Mutex<Option<...>> = Mutex::new(None);

pub fn init(boot_info: &mut BootInfo) {
    // store the P4 table in `P4_TABLE`
    P4_TABLE.store(boot_info.p4_table, Ordering::Relaxed);
    // println!("{:X}", boot_info as *const _ as usize);

    // setup_recursive_paging();
}

fn setup_recursive_paging() {
    let p4 = unsafe {
        P4_TABLE.load(Ordering::Relaxed)
            .as_mut()
            .expect("P4 table not loaded")
    };
    // println!("p4: {:?}", p4);
    use x86_64::registers::control::Cr3;

    let p4_frame = Cr3::read().0;

    // println!("P4 Frame: {:X}", p4_frame.start_address().as_u64());

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    // p4[511].set(p4_frame, flags);
}