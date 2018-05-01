use core::ptr::NonNull;
use alloc::allocator::{Alloc, Layout};
use ALLOCATOR;

use nabi::{Result, Error};

#[derive(Debug)]
pub struct Stack {
    ptr: NonNull<u8>,
    size: usize,
}

unsafe impl Send for Stack {}
unsafe impl Sync for Stack {}

impl Stack {
    /// Default stack alignment is 16 bytes.
    pub const ALIGN: usize = 16;

    fn layout(size: usize) -> Option<Layout> {
        Layout::from_size_align(size, Self::ALIGN).ok()
    }

    pub fn with_size(size: usize) -> Result<Stack> {
        let layout = Self::layout(size)
                .ok_or(Error::INTERNAL)?;
        let ptr = unsafe {
            let ptr = (&ALLOCATOR).alloc(layout)
                .map_err(|_| Error::NO_MEMORY)?;
            (ptr.as_ptr() as *mut u8).write_bytes(0, size);
            ptr
        }.cast();

        Ok(Stack {
            ptr,
            size,
        })
    }

    unsafe fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr() as _
    }

    pub fn top(&self) -> *mut u8 {
        unsafe { self.as_mut_ptr().add(self.size) }
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        unsafe {
            (&ALLOCATOR).dealloc(self.ptr.as_opaque(), Self::layout(self.size).unwrap());
        }
    }
}