
use x86_64::VirtAddr;
use x86_64::structures::paging::{Page, PageTableFlags, PageRangeInclusive};

use arch::paging::PageMapper;

use core::ops::{Deref, DerefMut};
use core::slice;

use nabi::Result;

extern "C" {
    fn erms_memset(dest: *mut u8, value: u8, size: usize);
}

bitflags! {
    pub struct MemFlags: u8 {
        const READ  = 1 << 0;
        const WRITE = 1 << 1;
        const EXEC  = 1 << 2;
    }
}

impl Into<PageTableFlags> for MemFlags {
    fn into(self) -> PageTableFlags {
        let mut flags = PageTableFlags::empty();

        if self.contains(MemFlags::READ) {
            flags |= PageTableFlags::PRESENT | PageTableFlags::GLOBAL;
        }
        if self.contains(MemFlags::WRITE) {
            flags |= PageTableFlags::WRITABLE;
        }
        if !self.contains(MemFlags::EXEC) {
            flags |= PageTableFlags::NO_EXECUTE;
        }

        flags
    }
}

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
    /// Convenience method to allocate a region directly from the Sip memory allocator
    pub fn allocate(size: usize) -> Option<Region> {
        super::SIP_ALLOCATOR.lock().allocate_region(size)
    }

    pub fn new(start: VirtAddr, size: usize, flags: MemFlags, zero: bool) -> Result<Self> {
        let mut region = Region {
            start,
            size,
            flags: flags.into(),
        };

        region.map(zero)
            .map_err(|_| internal_error!())?;

        Ok(region)
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

    fn pages(&self) -> PageRangeInclusive {
        let start_page = Page::containing_address(self.start);
        let end_page = Page::containing_address(self.start + self.size as u64 - 1 as u64);
        Page::range_inclusive(start_page, end_page)
    }

    fn map(&mut self, zero: bool) -> Result<()> {
        let mut mapper = unsafe { PageMapper::new() };

        for page in self.pages() {
            mapper.map(page, self.flags)
                .map_err(|_| internal_error!())?
                .flush();
        }

        if zero {
            debug_assert!(self.flags.contains(PageTableFlags::WRITABLE));
            unsafe {
                erms_memset(self.start().as_mut_ptr(), 0, self.size);
            }
        }
        Ok(())
    }

    fn unmap(&mut self) -> Result<()> {
        let mut mapper = unsafe { PageMapper::new() };

        for page in self.pages() {
            mapper.unmap(page)
                .map_err(|_| internal_error!())?
                .flush();
        }
        Ok(())
    }

    pub fn remap(&mut self, new_flags: MemFlags) -> Result<()> {
        let mut mapper = unsafe { PageMapper::new() };
        let new_flags = new_flags.into();

        for page in self.pages() {
            mapper.remap(page, new_flags)
                .map_err(|_| internal_error!())?
                .flush();
        }

        self.flags = new_flags;
        Ok(())
    }

    pub fn resize(&mut self, new_size: usize, zero: bool) -> Result<()> {
        let mut mapper = unsafe { PageMapper::new() };

        if new_size > self.size {
            let start_page = Page::containing_address(self.start + self.size as u64);
            let end_page = Page::containing_address(self.start + new_size as u64  - 1 as u64);
            for page in Page::range_inclusive(start_page, end_page) {
                if mapper.translate(page).is_none() {
                    mapper.map(page, self.flags)
                        .map_err(|_| internal_error!())?
                        .flush();
                }
            }

            if zero {
                debug_assert!(self.flags.contains(PageTableFlags::WRITABLE));
                unsafe {
                    erms_memset(self.start().as_mut_ptr(), 0, self.size);
                }
            }
        } else if new_size < self.size {
            let start_page = Page::containing_address(self.start + new_size as u64);
            let end_page = Page::containing_address(self.start + self.size as u64 - 1 as u64);
            for page in Page::range_inclusive(start_page, end_page) {
                if mapper.translate(page.clone()).is_some() {
                    mapper.unmap(page)
                        .map_err(|_| internal_error!())?
                        .flush();
                }
            }
        }

        self.size = new_size;

        Ok(())
    }

    pub fn contains(&self, addr: usize) -> bool {
        let addr = VirtAddr::new(addr as u64);
        addr >= self.start() && addr <= self.start() + self.size
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
        // ignore the result
        let _ = self.unmap();
    }
}
