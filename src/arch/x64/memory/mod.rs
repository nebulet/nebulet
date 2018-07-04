//! Blanket module for memory things
//! Allocator, paging (although there isn't much), etc

use os_bootinfo::BootInfo;
use x86_64::structures::paging::{PhysFrame, Size4KiB, FrameAllocator as PhysFrameAllocator, FrameDeallocator as PhysFrameDeallocator};

use arch::lock::IrqLock;
use self::bump::BumpAllocator;
use self::cache::FrameCache;

mod bump;
mod cache;

pub static FRAME_ALLOCATOR: IrqLock<Option<FrameCache<BumpAllocator>>> = IrqLock::new(None);

pub fn init(boot_info: &'static mut BootInfo) {
    *FRAME_ALLOCATOR.lock() = Some(FrameCache::new(BumpAllocator::new(&boot_info.memory_map)));
}

pub fn allocate_frame() -> Option<PhysFrame> {
    if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.allocate_frame()
    } else {
        panic!("frame allocator not initialized");
    }
}

pub fn deallocate_frame(frame: PhysFrame) {
    if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.deallocate_frame(frame)
    } else {
        panic!("frame allocator not initialized");
    }
}

pub struct GlobalFrameAllocator;

impl PhysFrameAllocator<Size4KiB> for GlobalFrameAllocator {
    fn alloc(&mut self) -> Option<PhysFrame<Size4KiB>> {
        allocate_frame()
    }
}

impl PhysFrameDeallocator<Size4KiB> for GlobalFrameAllocator {
    fn dealloc(&mut self, frame: PhysFrame<Size4KiB>) {
        deallocate_frame(frame);
    }
}

pub trait FrameAllocator {
    /// allocate `count` frames
    fn allocate_frame(&mut self) -> Option<PhysFrame>;
    /// deallocate `count` frames
    fn deallocate_frame(&mut self, frame: PhysFrame);
}
