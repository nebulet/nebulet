use x86_64::VirtAddr;
use x86_64::structures::paging::{Page, PageTableFlags};

use arch::paging::PageMapper;

pub use self::dlmalloc_rs::Allocator;

pub mod dlmalloc_rs;

unsafe fn map_heap(mapper: &mut PageMapper, offset: usize, size: usize) -> (*mut u8, usize) {
    let heap_start_page = Page::containing_address(VirtAddr::new(offset as u64));
    let heap_end_page = Page::containing_address(VirtAddr::new((offset + size) as u64));
    let flags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
    for page in Page::range(heap_start_page, heap_end_page) {
        mapper.map(page, flags)
            .expect("Couldn't map heap")
            .flush();
    }
    (heap_start_page.start_address().as_u64() as *mut u8, (heap_end_page - heap_start_page) as usize * 4096)
}
