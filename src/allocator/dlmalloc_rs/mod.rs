use alloc::heap::{Alloc, GlobalAlloc, Layout, AllocErr};
use core::cmp;
use core::ptr::{self, NonNull};
use core::alloc::Opaque;

use arch::lock::PreemptLock;

mod dlmalloc;

pub struct Dlmalloc(dlmalloc::Dlmalloc);

pub const DLMALLOC_INIT: Dlmalloc = Dlmalloc(dlmalloc::DLMALLOC_INIT);

mod sys;

unsafe impl Send for Dlmalloc {}

impl Dlmalloc {
    pub fn new() -> Dlmalloc {
        Dlmalloc(dlmalloc::Dlmalloc::new())
    }

    #[inline]
    pub unsafe fn malloc(&mut self, size: usize, align: usize) -> *mut u8 {
        if align <= self.0.malloc_alignment() {
            self.0.malloc(size)
        } else {
            self.0.memalign(align, size)
        }
    }

    #[inline]
    pub unsafe fn calloc(&mut self, size: usize, align: usize) -> *mut u8 {
        let ptr = self.malloc(size, align);
        if !ptr.is_null() && self.0.calloc_must_clear(ptr) {
            ptr::write_bytes(ptr, 0, size);
        }
        ptr
    }

    #[inline]
    pub unsafe fn free(&mut self, ptr: *mut u8, size: usize, align: usize) {
        drop((size, align));
        self.0.free(ptr)
    }

    #[inline]
    pub unsafe fn realloc(&mut self,
                          ptr: *mut u8,
                          old_size: usize,
                          old_align: usize,
                          new_size: usize) -> *mut u8 {
        if old_align <= self.0.malloc_alignment() {
            self.0.realloc(ptr, new_size)
        } else {
            let res = self.malloc(new_size, old_align);
            if !res.is_null() {
                let size = cmp::min(old_size, new_size);
                ptr::copy_nonoverlapping(ptr, res, size);
                self.free(ptr, old_size, old_align);
            }
            res
        }
    }
}

static HEAP: PreemptLock<Dlmalloc> = PreemptLock::new(DLMALLOC_INIT);

pub struct Allocator;

impl Allocator {
    pub unsafe fn init(offset: usize, size: usize) {}
}

unsafe impl<'a> Alloc for &'a Allocator {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<Opaque>, AllocErr> {
        let mut heap = HEAP.lock();
        let ptr = <Dlmalloc>::malloc(&mut heap, layout.size(), layout.align());
        if ptr.is_null() {
            Err(AllocErr)
        } else {
            Ok(NonNull::new_unchecked(ptr as _))
        }
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout)
        -> Result<NonNull<Opaque>, AllocErr>
    {
        let mut heap = HEAP.lock();
        let ptr = <Dlmalloc>::calloc(&mut heap, layout.size(), layout.align());
        if ptr.is_null() {
            Err(AllocErr)
        } else {
            Ok(NonNull::new_unchecked(ptr as _))
        }
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<Opaque>, layout: Layout) {
        let mut heap = HEAP.lock();
        <Dlmalloc>::free(&mut heap, ptr.as_ptr() as _, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(&mut self,
                      ptr: NonNull<Opaque>,
                      old_layout: Layout,
                      new_size: usize) -> Result<NonNull<Opaque>, AllocErr> {
        let mut heap = HEAP.lock();
        let ptr = <Dlmalloc>::realloc(
            &mut heap,
            ptr.as_ptr() as _,
            old_layout.size(),
            old_layout.align(),
            new_size,
        );


        if ptr.is_null() {
            Err(AllocErr)
        } else {
            Ok(NonNull::new_unchecked(ptr as _))
        }
    }

    #[inline]
    fn oom(&mut self) -> ! {
        <GlobalAlloc>::oom(*self);
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        let mut heap = HEAP.lock();
        <Dlmalloc>::calloc(&mut heap, layout.size(), layout.align()) as _
    }

    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout) {
        let mut heap = HEAP.lock();
        <Dlmalloc>::free(&mut heap, ptr as _, layout.size(), layout.align())
    }

    unsafe fn realloc(&self, ptr: *mut Opaque, layout: Layout, new_size: usize) -> *mut Opaque {
        let mut heap = HEAP.lock();
        <Dlmalloc>::realloc(
            &mut heap,
            ptr as _,
            layout.size(),
            layout.align(),
            new_size,
        ) as _
    }

    fn oom(&self) -> ! {
        panic!("Out of memory!");
    }
}