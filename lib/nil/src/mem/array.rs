use nabi::{Result, Error};
use alloc::heap::{Global, Layout};
use core::alloc::GlobalAlloc;
use core::ptr::{self, NonNull};
use core::mem;
use core::ops::{Deref, DerefMut};
use core::slice;
use mem::Bin;

pub struct Array<T> {
    backing: NonNull<T>,
    len: usize,
    capacity: usize,
}

unsafe impl<T: Sync + Send> Send for Array<T> {}
unsafe impl<T: Sync + Send> Sync for Array<T> {}

impl<T> Array<T> {
    pub fn new() -> Array<T> {
        Array {
            backing: NonNull::dangling(),
            len: 0,
            capacity: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Result<Array<T>> {
        if capacity == 0 {
            Ok(Self::new())
        } else {
            let layout = Layout::from_size_align(
                capacity * mem::size_of::<T>(),
                16
            ).map_err(|_| Error::INTERNAL)?;

            let ptr = unsafe {
                Global.alloc(layout)
            };

            let nonnull = NonNull::new(ptr)
                .ok_or(Error::NO_MEMORY)?
                .cast::<T>();
            
            Ok(Array {
                backing: nonnull,
                len: 0,
                capacity,
            })
        }
    }

    /// This returns the index of the pushed item.
    pub fn push(&mut self, item: T) -> Result<usize> {
        if self.len() == self.capacity() {
            self.double()?;
        }

        unsafe {
            self.backing.as_ptr()
                .add(self.len)
                .write(item);
        }

        self.len += 1;

        Ok(self.len - 1)
    }

    /// This pops an item off the end of the array.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.get_unchecked(self.len)))
            }
        }
    }

    pub fn replace_at(&mut self, index: usize, replacement: T) -> Option<T> {
        let loc = self.get_mut(index)?;
        Some(mem::replace(loc, replacement))
    }

    fn double(&mut self) -> Result<()> {
        if self.capacity == 0 {
            *self = Array::with_capacity(2)?;
            return Ok(());
        }

        let layout = Layout::from_size_align(
            self.capacity * mem::size_of::<T>(),
            16
        ).map_err(|_| Error::INTERNAL)?;

        let new_size = self.capacity * 2 * mem::size_of::<T>();

        let ptr = unsafe {
            Global.realloc(
                self.backing.as_opaque().as_ptr(),
                layout,
                new_size
            )
        };

        let nonnull = NonNull::new(ptr)
            .ok_or(Error::NO_MEMORY)?
            .cast::<T>();
        
        self.backing = nonnull;
        self.capacity = new_size;

        Ok(())
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Drop for Array<T> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            unsafe {
                let _ = Bin::from_nonnull(self.backing);
            }
        }
    }
}

impl<T> Deref for Array<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.backing.as_ptr(), self.len)
        }
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.backing.as_ptr(), self.len)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_push() {
        let mut arr = Array::new();
        arr.push(42).unwrap();
        assert_eq!(arr.pop().unwrap(), 42);
    }

    #[test]
    fn test_oom() {
        let res: Result<Array<u8>> = Array::with_capacity(usize::max_value() / 2);
        assert!(res.is_err());
    }
}
