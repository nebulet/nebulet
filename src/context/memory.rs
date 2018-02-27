use x86_64::structures::paging::{PageTableFlags, PageIter, Page};
use x86_64::VirtAddr;

use core::ptr;

use paging::ActivePageTable;

#[derive(Debug)]
pub struct Memory {
    start: VirtAddr,
    size: usize,
    flags: PageTableFlags,
}

impl Memory {
    pub fn new(start: VirtAddr, size: usize, flags: PageTableFlags, zero: bool) -> Memory {
        let mut memory = Memory {
            start: start,
            size: size,
            flags: flags,
        };

        memory.map(zero);

        memory
    }

    pub fn pages(&self) -> PageIter {
        let start_page = Page::containing_address(self.start);
        let end_page = Page::containing_address(self.start + self.size as u64 - 1);
        Page::range_inclusive(start_page, end_page)
    }

    pub fn map(&mut self, zero: bool) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for page in self.pages() {
            active_table.map(page, self.flags);
        }

        if zero {
            assert!(self.flags.contains(PageTableFlags::WRITABLE));
            unsafe {
                ptr::write_bytes(self.start.as_u64() as *mut u8, 0, self.size);
            }
        }
    }

    pub fn unmap(&mut self) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for page in self.pages() {
            active_table.unmap(page);
        }
    }

    pub fn start(&self) -> VirtAddr {
        self.start
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        self.unmap();
    }
}