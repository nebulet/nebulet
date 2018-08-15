pub use core::sync::atomic::Ordering;
use core::cell::UnsafeCell;
use core::mem::size_of;

union Transmute<T: Copy, U: Copy> {
    from: T,
    to: U,
}

const unsafe fn transmute_const<T: Copy, U: Copy>(from: T) -> U {
    Transmute::<T, U> { from }.to
}

macro_rules! call_atomic {
    ($func:ident, $($param:expr),*) => {
        if size_of::<T>() == size_of::<u8>() {
            $func::<T, u8>($($param,)*)
        } else if size_of::<T>() == size_of::<u16>() {
            $func::<T, u16>($($param,)*)
        } else if size_of::<T>() == size_of::<u32>() {
            $func::<T, u32>($($param,)*)
        } else if size_of::<T>() == size_of::<u64>() {
            $func::<T, u64>($($param,)*)
        } else {
            unimplemented!()
        }
    };
}

unsafe impl<T> Send for Atomic<T> {}
unsafe impl<T> Sync for Atomic<T> {}

#[derive(Debug)]
pub struct Atomic<T> {
    data: UnsafeCell<T>,
}

impl<T> Atomic<T>
where
    T: Copy
{
    #[inline]
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
        call_atomic!(atomic_load, self.data.get(), order)
    }

    #[inline]
    pub fn store(&self, val: T, order: Ordering) {
        call_atomic!(atomic_store, self.data.get(), val, order)
    }

    #[inline]
    pub fn swap(&self, val: T, order: Ordering) -> T {
        call_atomic!(atomic_swap, self.data.get(), val, order)
    }

    #[inline]
    pub fn compare_and_swap(&self, current: T, new: T, order: Ordering) -> T {
        call_atomic!(atomic_compare_and_swap, self.data.get(), current, new, order)   
    }

    #[inline]
    pub fn compare_exchange(&self, current: T, new: T, success: Ordering, failure: Ordering) -> Result<T, T> {
        call_atomic!(atomic_compare_exchange, self.data.get(), current, new, success, failure)
    }

    #[inline]
    pub fn compare_exchange_weak(&self, current: T, new: T, success: Ordering, failure: Ordering) -> Result<T, T> {
        call_atomic!(atomic_compare_exchange_weak, self.data.get(), current, new, success, failure)
    }
}

impl Atomic<bool> {
    #[inline]
    pub fn fetch_and(&self, val: bool, order: Ordering) -> bool {
        atomic_and::<bool, u8>(self.data.get(), val, order)
    }

    #[inline]
    pub fn fetch_nand(&self, val: bool, order: Ordering) -> bool {
        atomic_nand::<bool, u8>(self.data.get(), val, order)
    }

    #[inline]
    pub fn fetch_or(&self, val: bool, order: Ordering) -> bool {
        atomic_or::<bool, u8>(self.data.get(), val, order)
    }

    #[inline]
    pub fn fetch_xor(&self, val: bool, order: Ordering) -> bool {
        atomic_or::<bool, u8>(self.data.get(), val, order)
    }
}

macro_rules! atomic_integer_ops {
    ($($t:ty),*) => ($(
        impl Atomic<$t> {
            #[inline]
            pub fn fetch_add(&self, val: $t, order: Ordering) -> $t {
                atomic_fetch_add::<$t, $t>(self.data.get(), val, order)
            }

            #[inline]
            pub fn fetch_sub(&self, val: $t, order: Ordering) -> $t {
                atomic_fetch_sub::<$t, $t>(self.data.get(), val, order)
            }

            #[inline]
            pub fn fetch_and(&self, val: $t, order: Ordering) -> $t {
                atomic_and::<$t, $t>(self.data.get(), val, order)
            }

            #[inline]
            pub fn fetch_nand(&self, val: $t, order: Ordering) -> $t {
                atomic_nand::<$t, $t>(self.data.get(), val, order)
            }

            #[inline]
            pub fn fetch_or(&self, val: $t, order: Ordering) -> $t {
                atomic_or::<$t, $t>(self.data.get(), val, order)
            }

            #[inline]
            pub fn fetch_xor(&self, val: $t, order: Ordering) -> $t {
                atomic_xor::<$t, $t>(self.data.get(), val, order)
            }
        }
    )*);
}
    
atomic_integer_ops! { u8, i8, u16, i16, u32, i32, u64, i64, usize, isize }

impl<T> From<T> for Atomic<T>
where
    T: Copy
{
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

impl<T> Default for Atomic<T>
where
    T: Default + Copy
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[inline]
fn atomic_load<T: Copy, U: Copy>(dst: *mut T, order: Ordering) -> T {
    unsafe {
        transmute_const(sys::atomic_load(dst as *mut U, order))
    }
}

#[inline]
fn atomic_store<T: Copy, U: Copy>(dst: *mut T, val: T, order: Ordering) {
    unsafe {
        transmute_const(sys::atomic_store(dst as *mut U, transmute_const(val), order))
    }
}

#[inline]
fn atomic_swap<T: Copy, U: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    unsafe {
        transmute_const(sys::atomic_swap(dst as *mut U, transmute_const(val), order))
    }
}

#[inline]
fn atomic_compare_and_swap<T: Copy, U: Copy>(dst: *mut T, current: T, new: T, order: Ordering) -> T {
    match atomic_compare_exchange::<T, U>(dst, current, new, order, sys::strongest_failure_ordering(order)) {
        Ok(x) => x,
        Err(x) => x,
    }
}

#[inline]
fn atomic_compare_exchange<T: Copy, U: Copy>(dst: *mut T, current: T, new: T, success: Ordering, failure: Ordering) -> Result<T, T> {
    unsafe {
        match sys::atomic_compare_exchange(dst as *mut U, transmute_const(current), transmute_const(new), success, failure) {
            Ok(x) => Ok(transmute_const(x)),
            Err(x) => Err(transmute_const(x))
        }
    }
}

#[inline]
fn atomic_compare_exchange_weak<T: Copy, U: Copy>(dst: *mut T, current: T, new: T, success: Ordering, failure: Ordering) -> Result<T, T> {
    unsafe {
        match sys::atomic_compare_exchange_weak(dst as *mut U, transmute_const(current), transmute_const(new), success, failure) {
            Ok(x) => Ok(transmute_const(x)),
            Err(x) => Err(transmute_const(x)),
        }
    }
}

#[inline]
fn atomic_fetch_add<T: Copy, U: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    unsafe {
        transmute_const(sys::atomic_add(dst as *mut U, transmute_const(val), order))
    }
}

#[inline]
fn atomic_fetch_sub<T: Copy, U: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    unsafe {
        transmute_const(sys::atomic_sub(dst as *mut U, transmute_const(val), order))
    }
}

#[inline]
fn atomic_and<T: Copy, U: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    unsafe {
        transmute_const(sys::atomic_and(dst as *mut U, transmute_const(val), order))
    }
}

#[inline]
fn atomic_nand<T: Copy, U: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    unsafe {
        transmute_const(sys::atomic_nand(dst as *mut U, transmute_const(val), order))
    }
}

#[inline]
fn atomic_or<T: Copy, U: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    unsafe {
        transmute_const(sys::atomic_or(dst as *mut U, transmute_const(val), order))
    }
}

#[inline]
fn atomic_xor<T: Copy, U: Copy>(dst: *mut T, val: T, order: Ordering) -> T {
    unsafe {
        transmute_const(sys::atomic_xor(dst as *mut U, transmute_const(val), order))
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
            _ => panic!("invalid memory ordering"),
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
            _ => panic!("invalid memory ordering"),
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
            _ => panic!("invalid memory ordering"),
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
            _ => panic!("invalid memory ordering"),
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
            _ => panic!("invalid memory ordering"),
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
            _ => panic!("invalid memory ordering"),
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
            // (__Nonexhaustive, _) => panic!("invalid memory ordering"),
            // (_, __Nonexhaustive) => panic!("invalid memory ordering"),
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
            // (__Nonexhaustive, _) => panic!("invalid memory ordering"),
            // (_, __Nonexhaustive) => panic!("invalid memory ordering"),
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
            _ => panic!("invalid memory ordering"),
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
            _ => panic!("invalid memory ordering"),
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
            _ => panic!("invalid memory ordering"),
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
            _ => panic!("invalid memory ordering"),
        }
    }
}
