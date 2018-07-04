use core::any::{Any, TypeId};
use core::marker::Unsize;
use core::ops::{Deref, CoerceUnsized};
use core::sync::atomic::{self, AtomicUsize, Ordering};
use core::ptr::NonNull;
use core::{mem, ptr};
use core::alloc::Layout;
use mem::Bin;
use nabi::Result;
use object::dipatcher::Dispatcher;

struct RefInner<T: ?Sized> {
    count: AtomicUsize,
    data: T,
}

/// Reference counted ptr for
/// ensuring `KernelObject` lifetimes.
#[repr(transparent)]
#[derive(Debug)]
pub struct Ref<T: ?Sized> {
    ptr: NonNull<RefInner<T>>,
}

unsafe impl<T: ?Sized + Sync + Send> Send for Ref<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Ref<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Ref<U>> for Ref<T> {}

impl<T> Ref<T> {
    pub fn new(data: T) -> Result<Ref<T>> {
        let bin = Bin::new(RefInner {
            count: AtomicUsize::new(1),
            data,
        })?;

        Ok(Ref {
            ptr: bin.into_nonnull(),
        })
    }

    pub unsafe fn dangling() -> Ref<T> {
        Ref {
            ptr: NonNull::dangling(),
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

    pub fn into_raw(self: Self) -> *const T {
        let ptr: *const T = &*self;
        mem::forget(self);
        ptr
    }

    pub unsafe fn from_raw(ptr: *const T) -> Self {
        let align = mem::align_of_val(&*ptr);
        let layout = Layout::new::<RefInner<()>>();
        let offset = (layout.size() + layout.padding_needed_for(align)) as isize;

        let fake_ptr = ptr as *mut RefInner<T>;
        let ref_ptr = set_data_ptr(fake_ptr, (ptr as *mut u8).offset(-offset));

        Ref {
            ptr: NonNull::new_unchecked(ref_ptr),
        }
    }

    fn copy_ref(&self) -> Self {
        self.acquire();
        
        Self {
            ptr: self.ptr,
        }
    }

    pub fn acquire(&self) -> usize {
        self.inner().count.fetch_add(1, Ordering::Relaxed)
    }

    pub fn release(&self) -> usize {
        self.inner().count.fetch_sub(1, Ordering::Release)
    }

    pub fn refcount(&self) -> usize {
        self.inner().count.load(Ordering::Relaxed)
    }
}

impl Ref<Dispatcher> {
    pub fn cast<T: Dispatcher>(&self) -> Option<Ref<T>> {
        let self_: &Dispatcher = &**self;
        if self_.get_type_id() == TypeId::of::<T>() {
            let ptr: NonNull<RefInner<T>> = self.ptr.cast();
            let refptr = Ref { ptr, };
            refptr.acquire();
            Some(refptr)
        } else {
            None
        }
    }

    pub fn cast_ref<T: Dispatcher>(&self) -> Option<&T> {
        self.cast()
            .map(|refptr: Ref<T>| unsafe { &(&*refptr.ptr.as_ptr()).data })
    }
}

impl<T: ?Sized> Clone for Ref<T> {
    fn clone(&self) -> Ref<T> {
        self.copy_ref()
    }
}

unsafe impl<#[may_dangle] T: ?Sized> Drop for Ref<T> {
    fn drop(&mut self) {
        if self.release() != 1 {
            return;
        }

        atomic::fence(Ordering::Acquire);

        let ptr = self.ptr;

        unsafe {
            let _ = Bin::from_nonnull(ptr);

            atomic::fence(Ordering::Acquire);
        }     
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Ref<T> {
    #[inline]
    fn eq(&self, other: &Ref<T>) -> bool {
        let self_: &T = &*self;
        let other_: &T = &*other;
        self_ == other_
    }
}

impl<T: ?Sized> Deref for Ref<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.inner().data
    }
}

// Sets the data pointer of a `?Sized` raw pointer.
//
// For a slice/trait object, this sets the `data` field and leaves the rest
// unchanged. For a sized raw pointer, this simply sets the pointer.
unsafe fn set_data_ptr<T: ?Sized, U>(mut ptr: *mut T, data: *mut U) -> *mut T {
    ptr::write(&mut ptr as *mut _ as *mut *mut u8, data as *mut u8);
    ptr
}
