
use x86_64::{VirtAddr, PhysAddr};
use x86_64::structures::paging::{Page, PhysFrame, PageSize, Size4KiB,
    PageTableFlags, PageRangeInclusive, MapToError, UnmapError};

use arch::paging::PageMapper;
use arch::memory;

use core::ops::{Deref, DerefMut};
use core::slice;
use sync::atomic::{Atomic, Ordering};

use nabi::{Error, Result};

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
            match mapper.unmap(page) {
                Ok(mf) => mf.flush(),
                Err(UnmapError::PageNotMapped) => {},
                Err(_) => return Err(internal_error!()),
            }
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

    pub fn grow_from_phys_addr(&mut self, by: usize, phys_addr: usize) -> Result<()> {
        let mut mapper = unsafe { PageMapper::new() };

        let phys_addr = PhysAddr::new(phys_addr as u64);

        let start_page = Page::containing_address(self.start + self.size as u64);
        let end_page = Page::containing_address(self.start + self.size as u64 + by as u64);
        let start_frame = PhysFrame::containing_address(phys_addr);
        let end_frame = PhysFrame::containing_address(phys_addr + by as u64);

        let iter = Page::range(start_page, end_page)
            .zip(PhysFrame::range(start_frame, end_frame));

        for (page, frame) in iter {
            mapper.map_to(page, frame, self.flags)
                .map_err(|_| internal_error!())?
                .flush();
        }

        Ok(())
    }

    pub fn resize(&mut self, new_size: usize, zero: bool) -> Result<()> {
        let mut mapper = unsafe { PageMapper::new() };

        if new_size > self.size {
            let start_page = Page::containing_address(self.start + self.size as u64);
            let end_page = Page::containing_address(self.start + new_size as u64);
            for page in Page::range(start_page, end_page) {
                match mapper.map(page, self.flags) {
                    Ok(mf) => mf.flush(),
                    Err(MapToError::PageAlreadyMapped) => {},
                    Err(_) => return Err(internal_error!()),
                }
            }

            if zero {
                debug_assert!(self.flags.contains(PageTableFlags::WRITABLE));
                unsafe {
                    let start = self.start().as_mut_ptr::<u8>().add(self.size) as *mut u8;
                    erms_memset(start, 0, new_size - self.size);
                }
            }
        } else if new_size < self.size {
            let start_page = Page::containing_address(self.start + new_size as u64);
            let end_page = Page::containing_address(self.start + self.size as u64 - 1 as u64);
            for page in Page::range_inclusive(start_page, end_page) {
                match mapper.unmap(page) {
                    Ok(mf) => mf.flush(),
                    Err(UnmapError::PageNotMapped) => {},
                    Err(_) => return Err(internal_error!()),
                }
            }
        }

        self.size = new_size;

        Ok(())
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

/// Represents a region of virtual memory
/// that may or may not be currently mapped to 
/// physical memory. On accessing a lazily
/// mapped page, it will be mapped in.
#[derive(Debug)]
pub struct LazyRegion {
    start: VirtAddr,
    size: Atomic<usize>,
    flags: PageTableFlags,
}

impl LazyRegion {
    pub fn new(start: VirtAddr, size: usize, flags: MemFlags) -> Result<Self> {
        Ok(LazyRegion {
            start,
            size: Atomic::new(size),
            flags: flags.into(),
        })
    }

    #[inline]
    pub fn contains(&self, addr: *const ()) -> bool {
        let start = self.start.as_ptr::<u8>() as usize;
        let end = start + self.size.load(Ordering::Relaxed);

        (start..end).contains(&(addr as _))
    }

    /// Map a single 4096 byte page.
    pub fn map_page(&self, addr: *const ()) -> Result<()> {
        let mut mapper = unsafe { PageMapper::new() };
        
        let page = Page::containing_address(VirtAddr::new(addr as _));

        mapper.map(page, self.flags)
            .map_err(|_| internal_error!())?
            .flush();
        
        let page_ptr = page.start_address().as_mut_ptr();

        debug_assert!(self.flags.contains(PageTableFlags::WRITABLE));
        unsafe {
            erms_memset(page_ptr, 0, Size4KiB::SIZE as _);
        }

        Ok(())
    }

    pub fn map_range(&self, start: *const (), end: *const ()) -> Result<()> {
        let start_page = Page::containing_address(VirtAddr::new(start as _));
        let end_page = Page::containing_address(VirtAddr::new(end as _));

        let mut mapper = unsafe { PageMapper::new() };

        for page in Page::range_inclusive(start_page, end_page) {
            match mapper.map(page, self.flags) {
                Ok(mf) => {
                    mf.flush();
                    let page_ptr = page.start_address().as_mut_ptr();

                    debug_assert!(self.flags.contains(PageTableFlags::WRITABLE));
                    unsafe {
                        erms_memset(page_ptr, 0, Size4KiB::SIZE as _);
                    }
                },
                Err(MapToError::PageAlreadyMapped) => {},
                Err(_) => return Err(internal_error!()),
            }
        }

        Ok(())
    }

    pub fn unmap_range(&self, start: *const(), end: *const ()) -> Result<()> {
        let start_page = Page::containing_address(VirtAddr::new(start as _));
        let end_page = Page::containing_address(VirtAddr::new(end as _));

        let mut mapper = unsafe { PageMapper::new() };

        for page in Page::range_inclusive(start_page, end_page) {
            match mapper.unmap(page) {
                Ok(mf) => mf.flush(),
                Err(_) => return Err(internal_error!()),
            }
        }

        Ok(())
    }

    pub fn resize(&self, new_size: usize) -> Result<()> {
        self.size.store(new_size, Ordering::SeqCst);

        Ok(())
    }

    pub fn grow_from_phys_addr(&self, by: usize, phys_addr: usize) -> Result<()> {
        let mut mapper = unsafe { PageMapper::new() };

        let size = self.size.fetch_add(by, Ordering::SeqCst) as u64;

        let phys_addr = PhysAddr::new(phys_addr as u64);

        let start_page = Page::containing_address(self.start + size);
        let end_page = Page::containing_address(self.start + size + by as u64);
        let start_frame = PhysFrame::containing_address(phys_addr);
        let end_frame = PhysFrame::containing_address(phys_addr + by as u64);

        let iter = Page::range(start_page, end_page)
            .zip(PhysFrame::range(start_frame, end_frame));

        for (page, frame) in iter {
            mapper.map_to(page, frame, self.flags)
                .map_err(|_| internal_error!())?
                .flush();
        }

        Ok(())
    }

    pub fn grow_physically_contiguous(&self, by: usize) -> Result<PhysAddr> {
        let mut mapper = unsafe { PageMapper::new() };

        let range = memory::allocate_contiguous(by)
            .ok_or(Error::NO_RESOURCES)?;

        let physical_start = range.start.start_address();

        let size = self.size.fetch_add(by, Ordering::SeqCst) as u64;

        let start_page = Page::containing_address(self.start + size);
        let end_page = Page::containing_address(self.start + size + by as u64);

        let iter = Page::range(start_page, end_page)
            .zip(range);

        for (page, frame) in iter {
            mapper.map_to(page, frame, self.flags)
                .map_err(|_| internal_error!())?
                .flush();
        }

        Ok(physical_start)
    }

    fn pages(&self) -> PageRangeInclusive {
        let size = self.size.load(Ordering::Relaxed) as u64;
        let start_page = Page::containing_address(self.start);
        let end_page = Page::containing_address(self.start + size - 1 as u64);
        Page::range_inclusive(start_page, end_page)
    }

    fn unmap_all(&self) -> Result<()> {
        let mut mapper = unsafe { PageMapper::new() };

        for page in self.pages() {
            match mapper.unmap(page) {
                Ok(mf) => mf.flush(),
                Err(UnmapError::PageNotMapped) => {},
                Err(_) => return Err(internal_error!()),
            }
        }
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    pub fn start(&self) -> VirtAddr {
        self.start
    }
}

impl Deref for LazyRegion {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        let start = self.start.as_u64() as usize;
        let size = self.size.load(Ordering::Relaxed);
        unsafe { slice::from_raw_parts(start as *const u8, size) }
    }
}

impl DerefMut for LazyRegion {
    fn deref_mut(&mut self) -> &mut [u8] {
        let start = self.start.as_u64() as usize;
        let size = self.size.load(Ordering::Relaxed);
        unsafe { slice::from_raw_parts_mut(start as *mut u8, size) }
    }
}

impl Drop for LazyRegion {
    fn drop(&mut self) {
        let _ = self.unmap_all();
    }
}
