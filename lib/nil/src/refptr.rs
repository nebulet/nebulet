use core::any::{Any, TypeId};
use alloc::boxed::Box;
use core::marker::Unsize;
use core::ops::{Deref, CoerceUnsized};
use core::sync::atomic::{self, AtomicUsize, Ordering};
use core::ptr::{self, NonNull};
use alloc::heap::{Global, Layout};
use core::alloc::GlobalAlloc;

/// All kernel objects must implement this trait.
/// Kernel objects are intrusively refcounted.
pub trait KernelRef: Any + Send + Sync {}

impl<T: KernelRef> From<T> for Ref<T> {
    fn from(kref: T) -> Ref<T> {
        Ref::new(kref)
    }
}

struct RefInner<T: ?Sized> {
    count: AtomicUsize,
    data: T,
}

/// Reference counted ptr for
/// ensuring `KernelObject` lifetimes.
#[repr(transparent)]
pub struct Ref<T: ?Sized> {
    ptr: NonNull<RefInner<T>>,
}

unsafe impl<T: ?Sized + Sync + Send> Send for Ref<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Ref<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Ref<U>> for Ref<T> {}

impl<T> Ref<T> {
    pub fn new(data: T) -> Ref<T> {
        let boxed: Box<_> = box RefInner {
            count: AtomicUsize::new(1),
            data,
        };
        Ref {
            ptr: Box::into_raw_non_null(boxed),
        }
    }
}

impl<T: ?Sized> Ref<T> {
    #[inline]
    fn inner(&self) -> &RefInner<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[inline]
    pub fn ptr_eq(&self, other: &Ref<T>) -> bool {
        self.ptr == other.ptr
    }

    fn copy_ref(&self) -> Self {
        self.inc_ref();
        
        Self {
            ptr: self.ptr,
        }
    }

    pub fn inc_ref(&self) -> usize {
        self.inner().count.fetch_add(1, Ordering::Relaxed)
    }

    pub fn dec_ref(&self) -> usize {
        self.inner().count.fetch_sub(1, Ordering::Release)
    }
}

impl<T: KernelRef + ?Sized> Ref<T> {
    pub fn cast<U: KernelRef>(&self) -> Option<&U> {
        if TypeId::of::<T>() == TypeId::of::<U>() {
            let casted_ptr: NonNull<RefInner<U>> = self.ptr.cast();
            let data = unsafe { &((*casted_ptr.as_ptr()).data) };
            Some(&data)
        } else {
            None
        }
    }
}

impl<T: ?Sized> Clone for Ref<T> {
    fn clone(&self) -> Ref<T> {
        self.copy_ref()
    }
}

unsafe impl<#[may_dangle] T: ?Sized> Drop for Ref<T> {
    fn drop(&mut self) {
        if self.dec_ref() != 1 {
            return;
        }

        atomic::fence(Ordering::Acquire);

        let ptr = self.ptr;

        unsafe {
            ptr::drop_in_place(&mut self.ptr.as_mut().data);

            atomic::fence(Ordering::Acquire);

            Global.dealloc(ptr.as_opaque().as_ptr(), Layout::for_value(ptr.as_ref()));
        }     
    }
}

impl<T: ?Sized> PartialEq for Ref<T> {
    #[inline]
    fn eq(&self, other: &Ref<T>) -> bool {
        self.ptr_eq(other)
    }
}

impl<T: ?Sized> Deref for Ref<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.inner().data
    }
}
