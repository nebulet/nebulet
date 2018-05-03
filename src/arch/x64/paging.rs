use x86_64::structures::paging::{PageTable, PageTableFlags, RecursivePageTable, PhysFrame, Page, Mapper, Size4KB, MapperFlush, MapToError, UnmapError, FlagUpdateError};
use x86_64::ux::u9;

use arch::memory;

const P4: *mut PageTable = 0xffffffff_fffff000 as *mut _;
const RECURSIVE_PAGE_INDEX: u9 = u9::MAX;

pub unsafe fn init() -> PageMapper {
    PageMapper::new()
}

pub struct PageMapper {
    table: RecursivePageTable<'static>,
}

impl PageMapper {
    pub unsafe fn new() -> Self {
        PageMapper {
            table: RecursivePageTable::new_unchecked(&mut*P4, RECURSIVE_PAGE_INDEX),
        }
    }

    pub fn map(&mut self, page: Page<Size4KB>, flags: PageTableFlags) -> Result<MapperFlush<Size4KB>, MapToError> {
        let mut frame_allocator = || memory::allocate_frame();
        let frame = frame_allocator()
            .expect("Couldn't allocate any frames!");
        self.table.map_to(page, frame, flags, &mut frame_allocator)
    }

    pub fn unmap(&mut self, page: Page<Size4KB>) -> Result<MapperFlush<Size4KB>, UnmapError> {
        let mut frame_deallocator = |frame| memory::deallocate_frame(frame);
        self.table.unmap(page, &mut frame_deallocator)
    }

    pub fn remap(&mut self, page: Page<Size4KB>, flags: PageTableFlags) -> Result<MapperFlush<Size4KB>, FlagUpdateError> {
        self.table.update_flags(page, flags)
    }

    pub fn translate(&mut self, page: Page<Size4KB>) -> Option<PhysFrame> {
        self.table.translate(page)
    }
}
