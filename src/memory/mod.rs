//! Blanket module for memory things
//! Allocator, paging (although there isn't much), etc

use os_bootinfo::BootInfo;
use x86_64::structures::paging::{PageTable, PageTableFlags, PhysFrame, Level4};
use spin::Mutex;

use self::bump::BumpAllocator;
use interrupt;

pub mod bump;

type CurrentFrameAllocator = BumpAllocator;

pub static FRAME_ALLOCATOR: Mutex<Option<CurrentFrameAllocator>> = Mutex::new(None);

pub fn init(boot_info: &mut BootInfo) {
    setup_recursive_paging(boot_info.p4_table);

    *FRAME_ALLOCATOR.lock() = Some(BumpAllocator::new(boot_info.memory_map.clone()));
}

fn setup_recursive_paging(p4_table: &mut PageTable<Level4>) {
    use x86_64::registers::control::Cr3;

    let p4_frame = Cr3::read().0;

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    p4_table[511].set(p4_frame, flags);
}

pub fn allocate_frames(count: usize) -> Option<PhysFrame> {
    interrupt::disable_for(|| {
        if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
            allocator.allocate_frames(count)
        } else {
            panic!("frame allocator not initialized");
        }
    })
}

pub fn deallocate_frames(frame: PhysFrame, count: usize) {
    interrupt::disable_for(|| {
        if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
            allocator.deallocate_frames(frame, count)
        } else {
            panic!("frame allocator not initialized");
        }
    })
}

pub trait FrameAllocator {
    /// allocate `count` frames
    fn allocate_frames(&mut self, count: usize) -> Option<PhysFrame>;
    /// deallocate `count` frames
    fn deallocate_frames(&mut self, frame: PhysFrame, count: usize);
}