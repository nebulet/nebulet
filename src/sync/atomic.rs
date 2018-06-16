pub use core::sync::atomic::Ordering;
use core::cell::UnsafeCell;

pub trait AtomicOps
    where Self: Copy
{
    fn load(self: *mut Self, order: Ordering) -> Self;
    fn store(self: *mut Self, val: Self, order: Ordering);
    fn swap(self: *mut Self, val: Self, order: Ordering) -> Self;
    fn compare_and_swap(self: *mut Self, current: Self, new: Self, order: Ordering) -> Self;
    fn compare_exchange(self: *mut Self, current: Self, new: Self, success: Ordering, failure: Ordering) -> Result<Self, Self>;
    fn compare_exchange_weak(self: *mut Self, current: Self, new: Self, success: Ordering, failure: Ordering) -> Result<Self, Self>;
}

pub trait AtomicIntOps: AtomicOps {
    fn fetch_add(self: *mut Self, val: Self, order: Ordering) -> Self;
    fn fetch_sub(self: *mut Self, val: Self, order: Ordering) -> Self;
}

pub trait AtomicLogicalOps: AtomicOps {
    fn fetch_and(self: *mut Self, val: Self, order: Ordering) -> Self;
    fn fetch_nand(self: *mut Self, val: Self, order: Ordering) -> Self;
    fn fetch_or(self: *mut Self, val: Self, order: Ordering) -> Self;
    fn fetch_xor(self: *mut Self, val: Self, order: Ordering) -> Self;
}

unsafe impl<T: AtomicOps> Send for Atomic<T> {}
unsafe impl<T: AtomicOps> Sync for Atomic<T> {}

#[derive(Debug)]
pub struct Atomic<T: AtomicOps> {
    data: UnsafeCell<T>,
}

impl<T> Atomic<T> where T: AtomicOps {
    pub const fn new(val: T) -> Atomic<T> {
        Atomic {
            data: UnsafeCell::new(val),
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.data.get()
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    #[inline]
    pub fn load(&self, order: Ordering) -> T {
        T::load(self.data.get(), order)
    }

    #[inline]
    pub fn store(&self, val: T, order: Ordering) {
        T::store(self.data.get(), val, order);
    }

    #[inline]
    pub fn swap(&self, val: T, order: Ordering) -> T {
        T::swap(self.data.get(), val, order)
    }

    #[inline]
    pub fn compare_and_swap(&self, current: T, new: T, order: Ordering) -> T {
        T::compare_and_swap(self.data.get(), current, new, order)
    }

    #[inline]
    pub fn compare_exchange(&self, current: T, new: T, success: Ordering, failure: Ordering) -> Result<T, T> {
        T::compare_exchange(self.data.get(), current, new, success, failure)
    }

    #[inline]
    pub fn compare_exchange_weak(&self, current: T, new: T, success: Ordering, failure: Ordering) -> Result<T, T> {
        T::compare_exchange_weak(self.data.get(), current, new, success, failure)
    }
}

impl<T> Atomic<T> where T: AtomicIntOps {
    #[inline]
    pub fn fetch_add(&self, val: T, order: Ordering) -> T {
        T::fetch_add(self.data.get(), val, order)
    }

    #[inline]
    pub fn fetch_sub(&self, val: T, order: Ordering) -> T {
        T::fetch_sub(self.data.get(), val, order)
    }
}

impl<T> From<T> for Atomic<T> where T: AtomicOps {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

impl<T> Default for Atomic<T> where T: AtomicOps + Default {
    fn default() -> Self {
        Self::new(T::default())
    }
}

macro_rules! impl_atomic_ops {
    ($impl_type:ty) => {
        impl AtomicOps for $impl_type {
            #[inline]
            fn load(self: *mut Self, order: Ordering) -> Self {
                unsafe {
                    sys::atomic_load(self, order)
                }
            }

            #[inline]
            fn store(self: *mut Self, val: Self, order: Ordering) {
                unsafe {
                    sys::atomic_store(self, val, order);
                }
            }

            #[inline]
            fn swap(self: *mut Self, val: Self, order: Ordering) -> Self {
                unsafe {
                    sys::atomic_swap(self, val, order)
                }
            }

            #[inline]
            fn compare_and_swap(self: *mut Self, current: Self, new: Self, order: Ordering) -> Self {
                match self.compare_exchange(current, new, order, sys::strongest_failure_ordering(order)) {
                    Ok(x) => x,
                    Err(x) => x,
                }
            }

            #[inline]
            fn compare_exchange(self: *mut Self, current: Self, new: Self, success: Ordering, failure: Ordering) -> Result<Self, Self> {
                unsafe {
                    sys::atomic_compare_exchange(self, current, new, success, failure)
                }
            }

            #[inline]
            fn compare_exchange_weak(self: *mut Self, current: Self, new: Self, success: Ordering, failure: Ordering) -> Result<Self, Self> {
                unsafe {
                    sys::atomic_compare_exchange_weak(self, current, new, success, failure)
                }
            }
        }
    };
}

macro_rules! impl_atomic_ops_for_ptr {
    ($impl_type:ty) => {
        impl<T> AtomicOps for $impl_type {
            #[inline]
            fn load(self: *mut Self, order: Ordering) -> Self {
                unsafe {
                    sys::atomic_load(self as *mut usize, order) as Self
                }
            }

            #[inline]
            fn store(self: *mut Self, val: Self, order: Ordering) {
                unsafe {
                    sys::atomic_store(self as *mut usize, val as usize, order);
                }
            }

            #[inline]
            fn swap(self: *mut Self, val: Self, order: Ordering) -> Self {
                unsafe {
                    sys::atomic_swap(self as *mut usize, val as usize, order) as Self
                }
            }

            #[inline]
            fn compare_and_swap(self: *mut Self, current: Self, new: Self, order: Ordering) -> Self {
                match self.compare_exchange(current, new, order, sys::strongest_failure_ordering(order)) {
                    Ok(x) => x,
                    Err(x) => x,
                }
            }

            #[inline]
            fn compare_exchange(self: *mut Self, current: Self, new: Self, success: Ordering, failure: Ordering) -> Result<Self, Self> {
                unsafe {
                    match sys::atomic_compare_exchange(self as *mut usize, current as usize, new as usize, success, failure) {
                        Ok(x) => Ok(x as Self),
                        Err(x) => Err(x as Self),
                    }
                }
            }

            #[inline]
            fn compare_exchange_weak(self: *mut Self, current: Self, new: Self, success: Ordering, failure: Ordering) -> Result<Self, Self> {
                unsafe {
                    match sys::atomic_compare_exchange_weak(self as *mut usize, current as usize, new as usize, success, failure) {
                        Ok(x) => Ok(x as Self),
                        Err(x) => Err(x as Self),
                    }
                }
            }
        }
    };
}

macro_rules! impl_atomic_int_like {
    ($impl_type:ty) => {
        impl_atomic_logical!($impl_type);

        impl AtomicIntOps for $impl_type {
            #[inline]
            fn fetch_add(self: *mut Self, val: Self, order: Ordering) -> Self {
                unsafe {
                    sys::atomic_add(self, val, order)
                }
            }

            #[inline]
            fn fetch_sub(self: *mut Self, val: Self, order: Ordering) -> Self {
                unsafe {
                    sys::atomic_sub(self, val, order)
                }
            }
        }
    };
}

macro_rules! impl_atomic_logical {
    ($impl_type:ty) => {
        impl_atomic_ops!($impl_type);
        impl AtomicLogicalOps for $impl_type {
            #[inline]
            fn fetch_and(self: *mut Self, val: Self, order: Ordering) -> Self {
                unsafe {
                    sys::atomic_and(self, val, order)
                }
            }

            #[inline]
            fn fetch_nand(self: *mut Self, val: Self, order: Ordering) -> Self {
                unsafe {
                    sys::atomic_nand(self, val, order)
                }
            }

            #[inline]
            fn fetch_or(self: *mut Self, val: Self, order: Ordering) -> Self {
                unsafe {
                    sys::atomic_or(self, val, order)
                }
            }

            #[inline]
            fn fetch_xor(self: *mut Self, val: Self, order: Ordering) -> Self {
                unsafe {
                    sys::atomic_xor(self, val, order)
                }
            }
        }
    };
}

impl_atomic_int_like!(usize);
impl_atomic_int_like!(isize);
impl_atomic_int_like!(u32);
impl_atomic_int_like!(i32);
impl_atomic_int_like!(u16);
impl_atomic_int_like!(i16);
impl_atomic_int_like!(u8);
impl_atomic_int_like!(i8);

impl_atomic_ops_for_ptr!(*const T);
impl_atomic_ops_for_ptr!(*mut T);

impl AtomicOps for bool {
    #[inline]
    fn load(self: *mut Self, order: Ordering) -> Self {
        unsafe {
            sys::atomic_load(self as *mut u8, order) != 0
        }
    }

    #[inline]
    fn store(self: *mut Self, val: Self, order: Ordering) {
        unsafe {
            sys::atomic_store(self as *mut u8, val as u8, order);
        }
    }

    #[inline]
    fn swap(self: *mut Self, val: Self, order: Ordering) -> Self {
        unsafe {
            sys::atomic_swap(self as *mut u8, val as u8, order) != 0
        }
    }

    #[inline]
    fn compare_and_swap(self: *mut Self, current: Self, new: Self, order: Ordering) -> Self {
        match self.compare_exchange(current, new, order, sys::strongest_failure_ordering(order)) {
            Ok(x) => x,
            Err(x) => x,
        }
    }

    #[inline]
    fn compare_exchange(self: *mut Self, current: Self, new: Self, success: Ordering, failure: Ordering) -> Result<Self, Self> {
        unsafe {
            match sys::atomic_compare_exchange(self as *mut u8, current as u8, new as u8, success, failure) {
                Ok(x) => Ok(x != 0),
                Err(x) => Err(x != 0)
            }
        }
    }
    
    #[inline]
    fn compare_exchange_weak(self: *mut Self, current: Self, new: Self, success: Ordering, failure: Ordering) -> Result<Self, Self> {
        unsafe {
            match sys::atomic_compare_exchange_weak(self as *mut u8, current as u8, new as u8, success, failure) {
                Ok(x) => Ok(x != 0),
                Err(x) => Err(x != 0)
            }
        }
    }
}

impl AtomicLogicalOps for bool {
    #[inline]
    fn fetch_and(self: *mut Self, val: Self, order: Ordering) -> Self {
        unsafe {
            sys::atomic_and(self as *mut u8, val as u8, order) != 0
        }
    }

    #[inline]
    fn fetch_nand(self: *mut Self, val: Self, order: Ordering) -> Self {
        if val {
            self.fetch_xor(true, order)
        } else {
            bool::swap(self, true, order)
        }
    }

    #[inline]
    fn fetch_or(self: *mut Self, val: Self, order: Ordering) -> Self {
        unsafe {
            sys::atomic_or(self as *mut u8, val as u8, order) != 0
        }
    }

    #[inline]
    fn fetch_xor(self: *mut Self, val: Self, order: Ordering) -> Self {
        unsafe {
            sys::atomic_xor(self as *mut u8, val as u8, order) != 0
        }
    }
}

mod sys {
    use super::Ordering::{self, *};
    use core::intrinsics;

    #[inline]
    pub fn strongest_failure_ordering(order: Ordering) -> Ordering {
        match order {
            Release => Relaxed,
            Relaxed => Relaxed,
            SeqCst => SeqCst,
            Acquire => Acquire,
            AcqRel => Acquire,
            __Nonexhaustive => __Nonexhaustive,
        }
    }

    #[inline]
    pub unsafe fn atomic_store<T>(dst: *mut T, val: T, order: Ordering) {
        match order {
            Release => intrinsics::atomic_store_rel(dst, val),
            Relaxed => intrinsics::atomic_store_relaxed(dst, val),
            SeqCst => intrinsics::atomic_store(dst, val),
            Acquire => panic!("there is no such thing as an acquire store"),
            AcqRel => panic!("there is no such thing as an acquire/release store"),
            __Nonexhaustive => panic!("invalid memory ordering"),
        }
    }

    #[inline]
    pub unsafe fn atomic_load<T>(dst: *const T, order: Ordering) -> T {
        match order {
            Acquire => intrinsics::atomic_load_acq(dst),
            Relaxed => intrinsics::atomic_load_relaxed(dst),
            SeqCst => intrinsics::atomic_load(dst),
            Release => panic!("there is no such thing as a release load"),
            AcqRel => panic!("there is no such thing as an acquire/release load"),
            __Nonexhaustive => panic!("invalid memory ordering"),
        }
    }

    #[inline]
    pub unsafe fn atomic_swap<T>(dst: *mut T, val: T, order: Ordering) -> T {
        match order {
            Acquire => intrinsics::atomic_xchg_acq(dst, val),
            Release => intrinsics::atomic_xchg_rel(dst, val),
            AcqRel => intrinsics::atomic_xchg_acqrel(dst, val),
            Relaxed => intrinsics::atomic_xchg_relaxed(dst, val),
            SeqCst => intrinsics::atomic_xchg(dst, val),
            __Nonexhaustive => panic!("invalid memory ordering"),
        }
    }

    /// Returns the previous value (like __sync_fetch_and_add).
    #[inline]
    pub unsafe fn atomic_add<T>(dst: *mut T, val: T, order: Ordering) -> T {
        match order {
            Acquire => intrinsics::atomic_xadd_acq(dst, val),
            Release => intrinsics::atomic_xadd_rel(dst, val),
            AcqRel => intrinsics::atomic_xadd_acqrel(dst, val),
            Relaxed => intrinsics::atomic_xadd_relaxed(dst, val),
            SeqCst => intrinsics::atomic_xadd(dst, val),
            __Nonexhaustive => panic!("invalid memory ordering"),
        }
    }

    /// Returns the previous value (like __sync_fetch_and_sub).
    #[inline]
    pub unsafe fn atomic_sub<T>(dst: *mut T, val: T, order: Ordering) -> T {
        match order {
            Acquire => intrinsics::atomic_xsub_acq(dst, val),
            Release => intrinsics::atomic_xsub_rel(dst, val),
            AcqRel => intrinsics::atomic_xsub_acqrel(dst, val),
            Relaxed => intrinsics::atomic_xsub_relaxed(dst, val),
            SeqCst => intrinsics::atomic_xsub(dst, val),
            __Nonexhaustive => panic!("invalid memory ordering"),
        }
    }

    #[inline]
    pub unsafe fn atomic_compare_exchange<T>(dst: *mut T,
                                        old: T,
                                        new: T,
                                        success: Ordering,
                                        failure: Ordering)
                                        -> Result<T, T> {
        let (val, ok) = match (success, failure) {
            (Acquire, Acquire) => intrinsics::atomic_cxchg_acq(dst, old, new),
            (Release, Relaxed) => intrinsics::atomic_cxchg_rel(dst, old, new),
            (AcqRel, Acquire) => intrinsics::atomic_cxchg_acqrel(dst, old, new),
            (Relaxed, Relaxed) => intrinsics::atomic_cxchg_relaxed(dst, old, new),
            (SeqCst, SeqCst) => intrinsics::atomic_cxchg(dst, old, new),
            (Acquire, Relaxed) => intrinsics::atomic_cxchg_acq_failrelaxed(dst, old, new),
            (AcqRel, Relaxed) => intrinsics::atomic_cxchg_acqrel_failrelaxed(dst, old, new),
            (SeqCst, Relaxed) => intrinsics::atomic_cxchg_failrelaxed(dst, old, new),
            (SeqCst, Acquire) => intrinsics::atomic_cxchg_failacq(dst, old, new),
            (__Nonexhaustive, _) => panic!("invalid memory ordering"),
            (_, __Nonexhaustive) => panic!("invalid memory ordering"),
            (_, AcqRel) => panic!("there is no such thing as an acquire/release failure ordering"),
            (_, Release) => panic!("there is no such thing as a release failure ordering"),
            _ => panic!("a failure ordering can't be stronger than a success ordering"),
        };
        if ok { Ok(val) } else { Err(val) }
    }

    #[inline]
    pub unsafe fn atomic_compare_exchange_weak<T>(dst: *mut T,
                                            old: T,
                                            new: T,
                                            success: Ordering,
                                            failure: Ordering)
                                            -> Result<T, T> {
        let (val, ok) = match (success, failure) {
            (Acquire, Acquire) => intrinsics::atomic_cxchgweak_acq(dst, old, new),
            (Release, Relaxed) => intrinsics::atomic_cxchgweak_rel(dst, old, new),
            (AcqRel, Acquire) => intrinsics::atomic_cxchgweak_acqrel(dst, old, new),
            (Relaxed, Relaxed) => intrinsics::atomic_cxchgweak_relaxed(dst, old, new),
            (SeqCst, SeqCst) => intrinsics::atomic_cxchgweak(dst, old, new),
            (Acquire, Relaxed) => intrinsics::atomic_cxchgweak_acq_failrelaxed(dst, old, new),
            (AcqRel, Relaxed) => intrinsics::atomic_cxchgweak_acqrel_failrelaxed(dst, old, new),
            (SeqCst, Relaxed) => intrinsics::atomic_cxchgweak_failrelaxed(dst, old, new),
            (SeqCst, Acquire) => intrinsics::atomic_cxchgweak_failacq(dst, old, new),
            (__Nonexhaustive, _) => panic!("invalid memory ordering"),
            (_, __Nonexhaustive) => panic!("invalid memory ordering"),
            (_, AcqRel) => panic!("there is no such thing as an acquire/release failure ordering"),
            (_, Release) => panic!("there is no such thing as a release failure ordering"),
            _ => panic!("a failure ordering can't be stronger than a success ordering"),
        };
        if ok { Ok(val) } else { Err(val) }
    }

    #[inline]
    pub unsafe fn atomic_and<T>(dst: *mut T, val: T, order: Ordering) -> T {
        match order {
            Acquire => intrinsics::atomic_and_acq(dst, val),
            Release => intrinsics::atomic_and_rel(dst, val),
            AcqRel => intrinsics::atomic_and_acqrel(dst, val),
            Relaxed => intrinsics::atomic_and_relaxed(dst, val),
            SeqCst => intrinsics::atomic_and(dst, val),
            __Nonexhaustive => panic!("invalid memory ordering"),
        }
    }

    #[inline]
    pub unsafe fn atomic_nand<T>(dst: *mut T, val: T, order: Ordering) -> T {
        match order {
            Acquire => intrinsics::atomic_nand_acq(dst, val),
            Release => intrinsics::atomic_nand_rel(dst, val),
            AcqRel => intrinsics::atomic_nand_acqrel(dst, val),
            Relaxed => intrinsics::atomic_nand_relaxed(dst, val),
            SeqCst => intrinsics::atomic_nand(dst, val),
            __Nonexhaustive => panic!("invalid memory ordering"),
        }
    }

    #[inline]
    pub unsafe fn atomic_or<T>(dst: *mut T, val: T, order: Ordering) -> T {
        match order {
            Acquire => intrinsics::atomic_or_acq(dst, val),
            Release => intrinsics::atomic_or_rel(dst, val),
            AcqRel => intrinsics::atomic_or_acqrel(dst, val),
            Relaxed => intrinsics::atomic_or_relaxed(dst, val),
            SeqCst => intrinsics::atomic_or(dst, val),
            __Nonexhaustive => panic!("invalid memory ordering"),
        }
    }

    #[inline]
    pub unsafe fn atomic_xor<T>(dst: *mut T, val: T, order: Ordering) -> T {
        match order {
            Acquire => intrinsics::atomic_xor_acq(dst, val),
            Release => intrinsics::atomic_xor_rel(dst, val),
            AcqRel => intrinsics::atomic_xor_acqrel(dst, val),
            Relaxed => intrinsics::atomic_xor_relaxed(dst, val),
            SeqCst => intrinsics::atomic_xor(dst, val),
            __Nonexhaustive => panic!("invalid memory ordering"),
        }
    }
}
