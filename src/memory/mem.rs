use x86_64::{VirtAddr, PhysAddr};
use x86_64::structures::paging::{PhysFrame, PageIter, PageTableFlags, Page};
use arch::paging::ActivePageTable;

static MEMORY_FLAGS: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;

const WASM_PAGE_SIZE: usize = 1 << 16; // 64 KiB

/// The representation of a WebAssembly memory.
/// For now, max size of 4GiB.
/// 
/// `virtual_pages` and `physical_pages` are in
/// multiples of `WASM_PAGE_SIZE`.
#[derive(Debug)]
pub struct Memory {
    start: VirtAddr,
    virtual_pages: usize,
    physical_pages: usize,
}

impl Memory {
    pub const VIRT_PAGE_COUNT: usize = 1 << 14; // 4 gigabytes of pages

    /// This creates a memory with a max virtual size (which never changes)
    /// and a physical size, which is always less than or equal to the
    /// virtual size.
    pub fn new(start: VirtAddr, physical_page_count: usize) -> Memory {
        debug_assert!(physical_page_count <= VIRT_PAGE_COUNT);

        let mut memory = Memory {
            start,
            virtual_pages: VIRT_PAGE_COUNT,
            physical_pages: 0,
        };

        memory.grow(physical_page_count);

        mem
    }

    /// This expands the physically mapped memory by `by` (wasm) pages.
    /// 
    /// `by` is the number of pages (64KiBs each) to expand by
    pub fn grow(&mut self, by: usize) {
        debug_assert!(by + self.physical_pages <= self.virtual_pages);

        let by_size = by * WASM_PAGE_SIZE;

        let mut active_table = unsafe { ActivePageTable::new() };

        
    }

    pub fn map_to(&mut self, )
}

