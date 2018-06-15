use x86_64::structures::paging::{Size4KiB, PageSize};
use x86_64::VirtAddr;

use core::ops::{Deref, DerefMut};
use core::mem;

use memory::{LazyRegion, Region, MemFlags};

use nabi::Result;

/// Represents the entirety of the virtual memory that can be allocated to SIPs
///
/// This contains both code-memory, heap-memory, and guard-memory
pub struct SipAllocator {
    /// The end of available SIP memory
    end: usize,
    bump: usize,
}

impl SipAllocator {
    /// Create a new `AvailableSIPMemory`.
    pub const fn new(start: usize, end: usize) -> SipAllocator {
        SipAllocator {
            end,
            bump: start,
        }
    }

    /// Allocate a memory region of `size`.
    ///
    /// `size` will be rounded up to a multiple of 4KiB.
    pub(super) fn allocate_region(&mut self, size: usize) -> Option<Region> {
        let allocated_size = {
            let rem = size % Size4KiB::SIZE as usize;
            size + Size4KiB::SIZE as usize - rem
        };

        if self.bump + allocated_size > self.end {
            None
        } else {
            let virt_addr = VirtAddr::new(self.bump as u64);
            self.bump += allocated_size;
            let flags = MemFlags::READ | MemFlags::WRITE;
            Region::new(virt_addr, allocated_size, flags, true).ok()
        }
    }

    /// Allocate a `Memory`.
    fn allocate_wasm_memory(&mut self, pre_space: usize) -> Option<WasmMemory> {
        let pre_space = if pre_space != 0 {
            let rem = pre_space % Size4KiB::SIZE as usize;
            pre_space + Size4KiB::SIZE as usize - rem
        } else {
            0
        };

        let allocated_size = WasmMemory::DEFAULT_SIZE + pre_space; // 8 GiB

        if self.bump + allocated_size > self.end {
            None
        } else {
            let virt_addr = VirtAddr::new((self.bump + pre_space) as u64);

            let flags = MemFlags::READ | MemFlags::WRITE;

            let region = LazyRegion::new(virt_addr, 0, flags).ok()?;

            let pre_region = if pre_space != 0 {
                Some(Region::new(VirtAddr::new(self.bump as _), pre_space, flags, true).ok()?)
            } else {
                None
            };

            self.bump += allocated_size;

            Some(WasmMemory {
                region,
                total_size: WasmMemory::DEFAULT_SIZE,
                pre_region,
            })
        }
    }

    /// Allocate a `WasmStack` surrounded by two guard pages.
    fn allocate_stack(&mut self, size: usize) -> Option<WasmStack> {
        let requested_size = {
            let rem = size % Size4KiB::SIZE as usize;
            size + Size4KiB::SIZE as usize - rem
        };

        let allocated_size = requested_size + (Size4KiB::SIZE as usize * 2);

        if self.bump + allocated_size > self.end {
            None
        } else {
            let start = VirtAddr::new((self.bump as u64) + Size4KiB::SIZE);

            self.bump += allocated_size;

            let flags = MemFlags::READ | MemFlags::WRITE;
            let mut region = LazyRegion::new(start, requested_size, flags).ok()?;

            // Map in the last page of the stack.
            // This is a bit hacky, but it should prevent
            // page faults before the thread starts running.
            if region.size() >= Size4KiB::SIZE as _ {
                let addr = region.start() + region.size() as u64 - Size4KiB::SIZE;
                region.map_page(addr.as_ptr()).ok()?;
            }

            Some(WasmStack {
                region,
                total_size: allocated_size,
            })
        }
    }
}

/// This represents a WebAssembly Memory.
///
/// When this is dropped, the internal mapped region
/// will be unmapped.
#[derive(Debug)]
pub struct WasmMemory {
    pub region: LazyRegion,
    total_size: usize,
    pub pre_region: Option<Region>,
}

impl WasmMemory {
    pub const WASM_PAGE_SIZE: usize = 1 << 16; // 64 KiB
    pub const DEFAULT_HEAP_SIZE: usize = 1 << 32; // 4 GiB
    pub const DEFAULT_GUARD_SIZE: usize = 1 << 31; // 2 GiB
    pub const DEFAULT_SIZE: usize = Self::DEFAULT_HEAP_SIZE + Self::DEFAULT_GUARD_SIZE; // 8GiB

    pub fn allocate(pre_space: usize) -> Option<WasmMemory> {
        super::SIP_ALLOCATOR.lock().allocate_wasm_memory(pre_space)
    }

    /// Map virtual memory to physical memory by 
    /// multiples of `WasmMemory::WASM_PAGE_SIZE`.
    /// This starts at `mapped_end` and bump up.
    /// 
    /// Returns the number of pages before growing.
    pub fn grow(&mut self, count: usize) -> Result<usize> {
        let old_count = self.page_count();

        if count == 0 {
            return Ok(old_count);
        }

        let new_size = (old_count + count) * Self::WASM_PAGE_SIZE; 
        if new_size > self.total_size {
            Err(internal_error!())
        } else {
            self.region.resize(new_size)?;
            Ok(old_count)
        }
    }

    /// Map the specified region of physical memory to the next free part
    /// of the wasm linear memory.
    /// 
    /// Returns the offset of the mapped region in the wasm linear memory.
    pub fn physical_map(&mut self, phys_addr: u64, count: usize) -> Result<usize> {
        let old_count = self.page_count();

        let expand_by = count * Self::WASM_PAGE_SIZE;
        self.region.grow_from_phys_addr(expand_by, phys_addr as _)
            .map(|_| old_count * Self::WASM_PAGE_SIZE)
    }

    pub fn carve_slice(&self, offset: u32, size: u32) -> Option<&[u8]> {
        let start = offset as usize;
        let end = start + size as usize;
        let slice: &[u8] = &*self;

        if end <= self.mapped_size() {
            Some(&slice[start..end])
        } else {
            None
        }
    }

    pub fn carve_slice_mut(&mut self, offset: u32, size: u32) -> Option<&mut [u8]> {
        let start = offset as usize;
        let end = start + size as usize;
        let mapped_size = self.mapped_size();
        let slice: &mut [u8] = &mut *self;

        if end <= mapped_size {
            Some(&mut slice[start..end])
        } else {
            None
        }
    }

    pub fn carve<T>(&self, offset: u32) -> Option<&T> {
        let end_offset = offset as usize + mem::size_of::<T>();
        let mapped_size = self.mapped_size();

        if end_offset <= mapped_size {
            // in bounds
            unsafe {
                let start_ptr = self.start().as_ptr::<u8>();
                let ptr = start_ptr.add(offset as usize) as *const T;
                Some(&*ptr)
            }
        } else {
            None
        }
    }

    pub fn carve_mut<T>(&mut self, offset: u32) -> Option<&mut T> {
        let end_offset = offset as usize + mem::size_of::<T>();
        let mapped_size = self.mapped_size();

        if end_offset <= mapped_size {
            // in bounds
            unsafe {
                let start_ptr = self.start().as_mut_ptr::<u8>();
                let ptr = start_ptr.add(offset as usize) as *mut T;
                Some(&mut*ptr)
            }
        } else {
            None
        }
    }

    pub fn start(&self) -> VirtAddr {
        self.region.start()
    }

    pub fn unmapped_size(&self) -> usize {
        self.total_size - self.mapped_size()
    }

    pub fn mapped_size(&self) -> usize {
        self.region.size()
    }

    /// Returns the number of `WASM_PAGE_SIZE` pages
    /// currently mapped.
    pub fn page_count(&self) -> usize {
        self.mapped_size() / Self::WASM_PAGE_SIZE
    }

    pub fn in_mapped_bounds(&self, addr: *const ()) -> bool {
        let start_mapped = self.start().as_ptr::<u8>() as usize;
        let end_mapped = start_mapped + self.mapped_size();

        (start_mapped..end_mapped).contains(&(addr as _))
    }

    pub fn in_unmapped_bounds(&self, addr: *const ()) -> bool {
        let start_unmapped = self.start().as_ptr::<u8>() as usize + self.mapped_size();
        let end_unmapped = start_unmapped + self.unmapped_size();

        (start_unmapped..end_unmapped).contains(&(addr as _))
    }

    /// Map all the memory in the range [start_offset, end_offset).
    pub fn map_range(&mut self, start_offset: usize, end_offset: usize) -> Result<()> {
        let start = self.start().as_ptr::<u8>() as usize;
        let start_addr = start + start_offset;
        let end_addr = start + end_offset;

        self.region.map_range(start_addr as _, end_addr as _)
    }
}

impl Deref for WasmMemory {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &*self.region
    }
}

impl DerefMut for WasmMemory {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut *self.region
    }
}

#[derive(Debug)]
pub struct WasmStack {
    pub region: LazyRegion,
    /// Should be region.size + 8192 (two guard pages)
    total_size: usize,
}

impl WasmStack {
    pub fn allocate(size: usize) -> Option<WasmStack> {
        super::SIP_ALLOCATOR.lock().allocate_stack(size)
    }

    pub fn top(&self) -> *mut u8 {
        unsafe {
            (self.mapped_start().as_mut_ptr() as *mut u8).add(self.mapped_size())
        }
    }

    pub fn mapped_start(&self) -> VirtAddr {
        self.region.start()
    }

    pub fn unmapped_start(&self) -> VirtAddr {
        self.mapped_start() - Size4KiB::SIZE as u64
    }

    pub fn mapped_size(&self) -> usize {
        self.region.size()
    }

    pub fn total_size(&self) -> usize {
        self.total_size
    }

    pub fn addr_committed(&self, addr: *const ()) -> bool {
        let start = self.region.start().as_ptr::<u8>() as usize;
        let end = start + self.mapped_size();

        (start..end).contains(&(addr as _))
    }
}

impl Deref for WasmStack {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &*self.region
    }
}

impl DerefMut for WasmStack {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut *self.region
    }
}
