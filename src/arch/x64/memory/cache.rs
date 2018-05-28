/// Caches deallocated frames

use x86_64::structures::paging::PhysFrame;
use alloc::Vec;
use super::FrameAllocator;

pub struct FrameCache<T: FrameAllocator> {
    inner: T,
    freed: Vec<PhysFrame>
}

impl<T: FrameAllocator> FrameCache<T> {
    pub fn new(inner: T) -> FrameCache<T> {
        FrameCache {
            inner,
            freed: Vec::new(),
        }
    }
}

impl<T: FrameAllocator> FrameAllocator for FrameCache<T> {
    #[inline]
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.freed
            .pop()
            .or_else(|| self.inner.allocate_frame())
    }

    fn deallocate_frame(&mut self, frame: PhysFrame) {
        self.freed.push(frame);
    }
}
