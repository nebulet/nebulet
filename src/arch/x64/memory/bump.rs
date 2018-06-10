//! Bump Frame allocator
//! Much is borrowed from Redox OS and [Phil Opp's Blog](http://os.phil-opp.com/allocating-frames.html)

use x86_64::PhysAddr;
use x86_64::structures::paging::{PhysFrame, Size4KiB, PhysFrameRange};
use os_bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};

use super::FrameAllocator;

pub struct BumpAllocator {
    next_free_frame: PhysFrame<Size4KiB>,
    current_region: Option<MemoryRegion>,
    regions: &'static [MemoryRegion],
}

impl BumpAllocator {
    pub fn new(regions: &'static MemoryMap) -> BumpAllocator {
        let mut allocator = BumpAllocator {
            next_free_frame: PhysFrame::containing_address(PhysAddr::new(0)),
            current_region: None,
            regions,
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_region = self.regions.into_iter().find(|region| {
            let range: PhysFrameRange = region.range.into();
            region.region_type == MemoryRegionType::Usable
                && range.end > self.next_free_frame
        }).cloned();

        if let Some(region) = self.current_region {
            let range: PhysFrameRange = region.range.into();
            self.next_free_frame = range.start;
        }
    }
}

impl FrameAllocator for BumpAllocator {
    #[inline]
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        if let Some(region) = self.current_region {
            let found_frame = self.next_free_frame;

            // the last frame of the current region
            let range: PhysFrameRange = region.range.into();

            if found_frame >= range.end {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame += 1;
                return Some(found_frame);
            }
            // `frame` was not valid, try again with the updated `next_free_frame`
            self.allocate_frame()
        } else {
            None // no free frames left
        }
    }

    fn deallocate_frame(&mut self, _frame: PhysFrame) {
        // do nothing, leaky
    }
}
