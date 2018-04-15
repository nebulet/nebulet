use core::ptr::NonNull;
use alloc::allocator::{Alloc, Layout};
use ALLOCATOR;

use nabi::{Result, Error};

#[derive(Debug)]
pub struct Stack {
    ptr: NonNull<[u8; Stack::SIZE]>,
}

impl Stack {
    /// Default stack size is 1MiB.
    pub const SIZE: usize = 1 << 20;
    /// Default stack alignment is 16 bytes.
    pub const ALIGN: usize = 16;

    fn layout() -> Layout {
        unsafe { Layout::from_size_align_unchecked(Self::SIZE, Self::ALIGN) }
    }

    pub fn new() -> Result<Stack> {
        let ptr = unsafe {
            let ptr = (&ALLOCATOR).alloc(Self::layout())
                .map_err(|_| Error::NO_MEMORY)?;
            (ptr.as_ptr() as *mut u8).write_bytes(0, Self::SIZE);
            ptr
        };
        Ok(Stack { ptr: ptr.cast() })
    }

    unsafe fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr() as _
    }

    pub fn top(&self) -> *mut u8 {
        unsafe { self.as_mut_ptr().add(Self::SIZE) }
    }

    pub fn bottom(&self) -> *mut u8 {
        unsafe { self.as_mut_ptr() }
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        unsafe {
            (&ALLOCATOR).dealloc(NonNull::new_unchecked(self.as_mut_ptr() as _), Self::layout());
        }
    }
}