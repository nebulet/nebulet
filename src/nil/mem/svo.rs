use core::ptr::NonNull;
use core::mem;

struct FatData<T> {
    len: usize,
    ptr: NonNull<T>,
}

#[allow(unions_with_drop_fields)]
#[repr(C)]
union OneOrMany<T> {
    /// This is zero-sized, but
    /// it helps with making it
    /// look pretty.
    none: (),
    /// One item in the vec, stack allocated.
    one: T,
    /// Heap allocated.
    many: FatData<T>,
}

pub struct SmallVec<T> {
    capacity: usize,
    data: OneOrMany<T>,
}

impl<T> SmallVec<T> {
    pub fn new() -> SmallVec<T> {
        SmallVec {
            capacity: 0,
            data: OneOrMany { none: () },
        }
    }

    pub fn size(&self) -> usize {
        if self.is_stack_allocated() {
            self.stack_alloc_size()
        } else {
            unsafe { self.data.many.len }
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    fn stack_alloc_size(&self) -> usize {
        let size = mem::size_of::<usize>() * mem::size_of::<usize>();
        let mut cap = self.capacity;
        cap &= 1 << size - 2;
        cap >> size - 2
    }

    #[inline]
    fn is_stack_allocated(&self) -> bool {
        let size = mem::size_of::<usize>() * mem::size_of::<usize>();
        unsafe {
            mem::transmute((self.capacity >> (size - 1)) as u8)
        }
    }
}
