use x86_64::VirtAddr;
use x86_64::structures::paging::{Page, PageTableFlags};

use arch::paging::PageMapper;

#[cfg(feature = "linked_alloc")]
pub use self::linked_list::Allocator;

#[cfg(feature = "bump_alloc")]
pub use self::bump::Allocator;

// #[cfg(feature = "tree_alloc")]
// pub use self::tree::Allocator;

#[cfg(feature = "linked_alloc")]
mod linked_list;
#[cfg(feature = "bump_alloc")]
mod bump;
// mod tree;

unsafe fn map_heap(mapper: &mut PageMapper, offset: usize, size: usize) {
    let heap_start_page = Page::containing_address(VirtAddr::new(offset as u64));
    let heap_end_page = Page::containing_address(VirtAddr::new((offset + size - 1) as u64));
    let flags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        mapper.map(page, flags)
            .unwrap()
            .flush();
    }
}

pub unsafe fn init(mapper: &mut PageMapper) {
    let offset = ::KERNEL_HEAP_OFFSET;
    let size = ::KERNEL_HEAP_SIZE;

    // map heap pages
    map_heap(mapper, offset, size);

    // initialize global heap allocator
    Allocator::init(offset, size);
}