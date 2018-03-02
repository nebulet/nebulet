//! Blanket module for memory things
//! Allocator, paging (although there isn't much), etc

use os_bootinfo::BootInfo;
use x86_64::structures::paging::{PageTable, PageTableFlags, PhysFrame, Level4};
use spin::Mutex;

use self::bump::BumpAllocator;
use self::cache::FrameCache;
use interrupt;

mod bump;
mod cache;

pub static FRAME_ALLOCATOR: Mutex<Option<FrameCache<BumpAllocator>>> = Mutex::new(None);

pub fn init(boot_info: &mut BootInfo) {
    setup_recursive_paging(boot_info.p4_table);

    *FRAME_ALLOCATOR.lock() = Some(FrameCache::new(BumpAllocator::new(boot_info.memory_map.clone())));
}

fn setup_recursive_paging(p4_table: &mut PageTable<Level4>) {
    use x86_64::registers::control::Cr3;

    let p4_frame = Cr3::read().0;

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    p4_table[511].set(p4_frame, flags);
}

pub fn allocate_frame() -> Option<PhysFrame> {
    interrupt::disable_for(|| {
        if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
            allocator.allocate_frame()
        } else {
            panic!("frame allocator not initialized");
        }
    })
}

pub fn deallocate_frame(frame: PhysFrame) {
    interrupt::disable_for(|| {
        if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
            allocator.deallocate_frame(frame)
        } else {
            panic!("frame allocator not initialized");
        }
    })
}

pub trait FrameAllocator {
    /// allocate `count` frames
    fn allocate_frame(&mut self) -> Option<PhysFrame>;
    /// deallocate `count` frames
    fn deallocate_frame(&mut self, frame: PhysFrame);
}