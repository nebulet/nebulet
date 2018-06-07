use x86_64::structures::paging::{Size4KB, PageSize};
use x86_64::VirtAddr;

use core::ops::{Deref, DerefMut};
use core::mem;

use memory::{Region, MemFlags};

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
            let rem = size % Size4KB::SIZE as usize;
            size + Size4KB::SIZE as usize - rem
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
    fn allocate_wasm_memory(&mut self) -> Option<WasmMemory> {
        let allocated_size = WasmMemory::DEFAULT_SIZE; // 8 GiB

        if self.bump + allocated_size > self.end {
            None
        } else {
            let virt_addr = VirtAddr::new(self.bump as u64);

            self.bump += allocated_size;

            WasmMemory::new(virt_addr).ok()
        }
    }

    /// Allocate a `WasmStack` surrounded by two guard pages.
    fn allocate_stack(&mut self, size: usize) -> Option<WasmStack> {
        let requested_size = {
            let rem = size % Size4KB::SIZE as usize;
            size + Size4KB::SIZE as usize - rem
        };

        let allocated_size = requested_size + (Size4KB::SIZE as usize * 2);

        if self.bump + allocated_size > self.end {
            None
        } else {
            let start = VirtAddr::new((self.bump as u64) + Size4KB::SIZE);

            self.bump += allocated_size;

            let flags = MemFlags::READ | MemFlags::WRITE;
            let region = Region::new(start, requested_size, flags, true).ok()?;

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
    region: Region,
    total_size: usize,
}

impl WasmMemory {
    pub const WASM_PAGE_SIZE: usize = 1 << 16; // 64 KiB
    pub const DEFAULT_HEAP_SIZE: usize = 1 << 32; // 4 GiB
    pub const DEFAULT_GUARD_SIZE: usize = 1 << 32; // 4 GiB
    pub const DEFAULT_SIZE: usize = Self::DEFAULT_HEAP_SIZE + Self::DEFAULT_GUARD_SIZE; // 8GiB

    pub fn allocate() -> Option<WasmMemory> {
        super::SIP_ALLOCATOR.lock().allocate_wasm_memory()
    }

    /// Create a completely unmapped `Memory` with unmapped size of `size`.
    /// The mapped size to start is `0`.
    pub fn with_capacity(start: VirtAddr, unmapped_size: usize, mapped_size: usize) -> Result<Self> {
        let flags = MemFlags::READ | MemFlags::WRITE;
        let region = Region::new(start, mapped_size, flags, true)?;

        Ok(WasmMemory {
            region,
            total_size: unmapped_size + mapped_size,
        })
    }

    /// Create a new `Memory` with an unmapped size of 4 + 2 GiB and a mapped size of `0`.
    pub fn new(start: VirtAddr) -> Result<Self> {
        Self::with_capacity(start, Self::DEFAULT_SIZE, 0)
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
            self.region.resize(new_size, true)?;
            Ok(old_count)
        }
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
    region: Region,
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
        self.mapped_start() - Size4KB::SIZE as u64
    }

    pub fn mapped_size(&self) -> usize {
        self.region.size()
    }

    pub fn total_size(&self) -> usize {
        self.total_size
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
