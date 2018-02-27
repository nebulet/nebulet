//! Bump Frame allocator
//! Much is borrowed from Redox OS and [Phil Opp's Blog](http://os.phil-opp.com/allocating-frames.html)

use x86_64::PhysAddr;
use x86_64::structures::paging::{PhysFrame, PAGE_SIZE};
use os_bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};

use super::FrameAllocator;
use macros::println;

pub struct BumpAllocator {
    next_free_frame: PhysFrame,
    current_region: Option<MemoryRegion>,
    regions: MemoryMap,
}

impl BumpAllocator {
    pub fn new(memory_map: MemoryMap) -> BumpAllocator {
        let mut allocator = BumpAllocator {
            // start at two frames from 0
            next_free_frame: PhysFrame::containing_address(PhysAddr::new(4096 * 2)),
            current_region: None,
            regions: memory_map,
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_region = self.regions.clone().into_iter().find(|region| {
            // let address = region.start_addr + region.len - 1;
            PhysFrame::containing_address(region.start_addr + region.len - 1) >= self.next_free_frame
                && region.region_type == MemoryRegionType::Usable
        });

        if let Some(region) = self.current_region {
            let start_frame = PhysFrame::containing_address(region.start_addr);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

impl FrameAllocator for BumpAllocator {
    fn allocate_frames(&mut self, count: usize) -> Option<PhysFrame> {
        if count == 0 {
            None
        } else if let Some(region) = self.current_region {
            let start_frame = self.next_free_frame.clone();
            let end_frame = self.next_free_frame.clone() + count as u64 - 1;

            // the last frame of the current region
            let current_region_last_frame = {
                let address = region.start_addr + region.len - 1;
                PhysFrame::containing_address(address)
            };

            if end_frame > current_region_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame += count as u64;
                return Some(start_frame);
            }
            // `frame` was not valid, try again with the updated `next_free_frame`
            self.allocate_frames(count)
        } else {
            None // no free frames left
        }
    }

    fn deallocate_frames(&mut self, _frame: PhysFrame, _count: usize) {
        // do nothing, leaky
    }
}