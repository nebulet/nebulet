//! Bump Frame allocator
//! Much is borrowed from Redox OS and [Phil Opp's Blog](http://os.phil-opp.com/allocating-frames.html)

use x86_64::PhysAddr;
use x86_64::structures::paging::{PhysFrame, Size4KiB, PhysFrameRange};
use bootloader::bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};

use super::FrameAllocator;

pub struct BumpAllocator {
    next_free_frame: PhysFrame<Size4KiB>,
    current_region: Option<MemoryRegion>,
    physical_pool: MemoryRegion,
    regions: [MemoryRegion; 32],
}

impl BumpAllocator {
    pub fn new(memory_map: &'static MemoryMap, physical_pool_size: usize) -> BumpAllocator {
        debug_assert!(memory_map.len() <= 32);

        let mut regions = [MemoryRegion::empty(); 32];

        regions[..memory_map.len()].copy_from_slice(memory_map);

        let mut allocator = BumpAllocator {
            next_free_frame: PhysFrame::containing_address(PhysAddr::new(0)),
            current_region: None,
            physical_pool: MemoryRegion::empty(),
            regions,
        };

        allocator.fill_physical_pool(physical_pool_size);
        allocator.choose_next_area();
        allocator
    }

    fn fill_physical_pool(&mut self, size: usize) {
        if let Some(region) = self.regions.iter_mut().find(|region| region.range.end.start_address() - region.range.start.start_address() >= size as u64) {
            let frame_size = region.range.start.size();

            let new_region_end = region.range.start + (((size as u64) + frame_size) / frame_size);

            self.physical_pool = MemoryRegion {
                range: PhysFrame::range(
                    region.range.start,
                    new_region_end,
                ),
                region_type: region.region_type,
            };

            region.range.start = new_region_end;
        }
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

    #[inline]
    fn deallocate_frame(&mut self, _frame: PhysFrame) {
        // do nothing, leaky
    }

    #[inline]
    fn allocate_contiguous(&mut self, size: usize) -> Option<PhysFrameRange> {
        let frame_size = self.physical_pool.range.start.size();
        let frame_count = ((size as u64) + frame_size) / frame_size;

        if self.physical_pool.range.start + frame_count < self.physical_pool.range.end {
            let old_start = self.physical_pool.range.start;

            self.physical_pool.range.start += frame_count;

            Some(PhysFrame::range(
                old_start,
                self.physical_pool.range.start,
            ))
        } else {
            None
        }
    }
    
    #[inline]
    fn deallocate_contiguous(&mut self, _range: PhysFrameRange) {
        // do nothing
    }
}
