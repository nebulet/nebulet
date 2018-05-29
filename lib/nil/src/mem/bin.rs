use core::ptr::{NonNull, drop_in_place};
use core::mem;
use core::ops::{Deref, DerefMut, CoerceUnsized};
use core::marker::Unsize;
use nabi::{Result, Error};
use alloc::heap::{Global, Layout};
use core::alloc::GlobalAlloc;

pub struct Bin<T: ?Sized> {
    ptr: NonNull<T>,
}

unsafe impl<T: ?Sized + Sync + Send> Send for Bin<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Bin<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Bin<U>> for Bin<T> {}

impl<T> Bin<T> {
    pub fn new(data: T) -> Result<Bin<T>> {
        let layout = Layout::from_size_align(mem::size_of::<T>(), 16)
            .map_err(|_| Error::INTERNAL)?;
        
        let ptr = unsafe {
            Global.alloc(layout)
        };

        let nonnull = NonNull::new(ptr)
            .ok_or(Error::NO_MEMORY)?
            .cast::<T>();

        unsafe {
            nonnull.as_ptr().write(data);
        }

        Ok(Bin {
            ptr: nonnull,
        })
    }
}

impl<T: ?Sized> Bin<T> {
    pub fn into_nonnull(self) -> NonNull<T> {
        let ptr = self.ptr;
        mem::forget(self);
        ptr
    }

    pub unsafe fn from_nonnull(ptr: NonNull<T>) -> Bin<T> {
        Bin {
            ptr,
        }
    }
}

impl<T: ?Sized> Deref for Bin<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            self.ptr.as_ref()
        }
    }
}

impl<T: ?Sized> DerefMut for Bin<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            self.ptr.as_mut()
        }
    }
}

impl<T: ?Sized> Drop for Bin<T> {
    fn drop(&mut self) {
        let ptr = self.ptr;

        unsafe {
            drop_in_place(ptr.as_ptr());
            
            Global.dealloc(ptr.as_opaque().as_ptr(), Layout::for_value(self));
        }
    }
}
