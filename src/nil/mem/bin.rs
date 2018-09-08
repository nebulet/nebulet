use alloc::alloc::{Global, Layout};
use core::alloc::Alloc;
use core::marker::Unsize;
use core::mem;
use core::ops::{CoerceUnsized, Deref, DerefMut};
use core::ptr::{drop_in_place, NonNull};
use nabi::{Error, Result};

pub struct Bin<T: ?Sized> {
    ptr: NonNull<T>,
}

unsafe impl<T: ?Sized + Sync + Send> Send for Bin<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Bin<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Bin<U>> for Bin<T> {}

impl<T> Bin<T> {
    pub fn new(data: T) -> Result<Bin<T>> {
        let layout =
            Layout::from_size_align(mem::size_of::<T>(), 16).map_err(|_| Error::INTERNAL)?;

        let ptr_res = unsafe { Global.alloc(layout) };

        let ptr = ptr_res.map_err(|_| Error::NO_MEMORY)?.cast::<T>();

        unsafe {
            ptr.as_ptr().write(data);
        }

        Ok(Bin { ptr })
    }
}

impl<T: ?Sized> Bin<T> {
    pub fn into_nonnull(self) -> NonNull<T> {
        let ptr = self.ptr;
        mem::forget(self);
        ptr
    }

    pub unsafe fn from_nonnull(ptr: NonNull<T>) -> Bin<T> {
        Bin { ptr }
    }
}

impl<T: ?Sized> Deref for Bin<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Bin<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: ?Sized> Drop for Bin<T> {
    fn drop(&mut self) {
        let ptr = self.ptr;

        unsafe {
            drop_in_place(ptr.as_ptr());

            Global.dealloc(ptr.cast(), Layout::for_value(self));
        }
    }
}
