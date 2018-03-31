
use x86_64::VirtAddr;
use x86_64::structures::paging::{Page, PageIter, PAGE_SIZE, PageTableFlags};

use arch::paging::ActivePageTable;

use core::ops::{Deref, DerefMut};
use core::slice;

/// Represents any region of memory that needs to be mapped/unmapped/remapped
/// 
/// Derefs to a slice that contains the memory to which this refers.
#[derive(Debug)]
pub struct Region {
    start: VirtAddr,
    size: usize,
    flags: PageTableFlags,
}

impl Region {
    pub fn new(start: VirtAddr, size: usize, flags: PageTableFlags, zero: bool) -> Self {
        let mut region = Region {
            start,
            size,
            flags,
        };

        region.map(zero);

        region
    }

    pub fn start(&self) -> VirtAddr {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn flags(&self) -> PageTableFlags {
        self.flags
    }

    fn pages(&self) -> PageIter {
        let start_page = Page::containing_address(self.start);
        let end_page = Page::containing_address(self.start + self.size as u64 - 1);
        Page::range_inclusive(start_page, end_page)
    }

    fn map(&mut self, zero: bool) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for page in self.pages() {
            active_table.map(page, self.flags)
                .flush(&mut active_table);
        }

        if zero {
            debug_assert!(self.flags.contains(PageTableFlags::WRITABLE));
            unsafe {
                (self.start.as_u64() as *mut u8).write_bytes(0, self.size);
            }
        }
    }

    fn unmap(&mut self) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for page in self.pages() {
            active_table.unmap(page)
                .flush(&mut active_table);
        }
    }

    pub fn remap(&mut self, new_flags: PageTableFlags) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for page in self.pages() {
            active_table.remap(page, new_flags)
                .flush(&mut active_table);
        }

        self.flags = new_flags;
    }

    pub fn resize(&mut self, new_size: usize, zero: bool) {
        let mut active_table = unsafe { ActivePageTable::new() };

        if new_size > self.size {
            let start_page = Page::containing_address(self.start + self.size as u64);
            let end_page = Page::containing_address(self.start + new_size as u64 - 1);
            for page in Page::range_inclusive(start_page, end_page) {
                if active_table.translate_page(page.clone()).is_none() {
                    active_table.map(page, self.flags)
                        .flush(&mut active_table);
                }
            }

            if zero {
                debug_assert!(self.flags.contains(PageTableFlags::WRITABLE));
                unsafe {
                    (self.start.as_u64() as *mut u8).write_bytes(0, self.size);
                }
            }
        } else if new_size < self.size {
            let start_page = Page::containing_address(self.start + new_size as u64);
            let end_page = Page::containing_address(self.start + self.size as u64 - 1);
            for page in Page::range_inclusive(start_page, end_page) {
                if active_table.translate_page(page.clone()).is_some() {
                    active_table.unmap(page)
                        .flush(&mut active_table);
                }
            }
        }

        self.size = new_size;
    }
}

impl Deref for Region {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        let start = self.start.as_u64() as usize;
        let len = self.size;
        unsafe { slice::from_raw_parts(start as *const u8, len) }
    }
}

impl DerefMut for Region {
    fn deref_mut(&mut self) -> &mut [u8] {
        let start = self.start.as_u64() as usize;
        let len = self.size;
        unsafe { slice::from_raw_parts_mut(start as *mut u8, len) }
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        self.unmap();
    }
}