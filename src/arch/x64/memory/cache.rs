/// Caches deallocated frames

use x86_64::structures::paging::{PhysFrame, PhysFrameRange};
use alloc::vec::Vec;
use super::FrameAllocator;

pub struct FrameCache<T: FrameAllocator> {
    inner: T,
    freed_frames: Vec<PhysFrame>,
    physical_pool: Vec<PhysFrameRange>,
}

impl<T: FrameAllocator> FrameCache<T> {
    pub fn new(inner: T) -> FrameCache<T> {
        FrameCache {
            inner,
            freed_frames: Vec::new(),
            physical_pool: Vec::new(),
        }
    }
}

impl<T: FrameAllocator> FrameAllocator for FrameCache<T> {
    #[inline]
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.freed_frames
            .pop()
            .or_else(|| self.inner.allocate_frame())
    }

    #[inline]
    fn deallocate_frame(&mut self, frame: PhysFrame) {
        self.freed_frames.push(frame);
    }

    #[inline]
    fn allocate_contiguous(&mut self, size: usize) -> Option<PhysFrameRange> {
        self.physical_pool.iter_mut().enumerate().find_map(|(i, region)| {
            println!("debug {}:{}", file!(), line!());
            if region.end.start_address() - region.start.start_address() >= size as u64 {
                Some(i)
            } else {
                None
            }
        }).map(|index| self.physical_pool.swap_remove(index))
          .or_else(|| {
              println!("debug {}:{}", file!(), line!());
              self.inner.allocate_contiguous(size)
          })
    }
    
    #[inline]
    fn deallocate_contiguous(&mut self, range: PhysFrameRange) {
        self.physical_pool.push(range);
    }
}
