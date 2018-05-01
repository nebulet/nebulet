use core::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut, Drop};

use arch::cpu;
use arch::interrupt;

#[derive(Debug)]
pub struct Spinlock<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct SpinGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}

unsafe impl<T: ?Sized + Send> Sync for Spinlock<T> {}
unsafe impl<T: ?Sized + Send> Send for Spinlock<T> {}

impl<T> Spinlock<T> {
    pub const fn new(data: T) -> Spinlock<T> {
        Spinlock {
            lock: ATOMIC_BOOL_INIT,
            data: UnsafeCell::new(data),
        }
    }

    fn obtain_lock(&self) {
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) != false {
            while self.lock.load(Ordering::Relaxed) {
                interrupt::pause();
            }
        }
    }

    pub fn lock(&self) -> SpinGuard<T> {
        self.obtain_lock();

        SpinGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    pub fn try_lock(&self) -> Option<SpinGuard<T>> {
        if self.lock.compare_and_swap(false, true, Ordering::Acquire) == false {
            Some(SpinGuard {
                lock: &self.lock,
                data: unsafe { &mut *self.data.get() }
            })
        } else {
            None
        }
    }
}

impl<T: ?Sized + Default> Default for Spinlock<T> {
    fn default() -> Spinlock<T> {
        Spinlock::new(Default::default())
    }
}

impl<'a, T: ?Sized> Deref for SpinGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.data
    }
}

impl<'a, T: ?Sized> DerefMut for SpinGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.data
    }
}

impl<'a, T: ?Sized> Drop for SpinGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}

pub struct PreemptLock<T: ?Sized> {
    data: UnsafeCell<T>,
}

pub struct PreemptGuard<'a, T: ?Sized + 'a> {
    data: &'a mut T,
}

unsafe impl<T: ?Sized + Send> Sync for PreemptLock<T> {}
unsafe impl<T: ?Sized + Send> Send for PreemptLock<T> {}

impl<T> PreemptLock<T> {
    pub const fn new(data: T) -> PreemptLock<T> {
        PreemptLock {
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> PreemptGuard<T> {
        unsafe {
            cpu::preempt::disable();
        }
        PreemptGuard {
            data: unsafe { &mut *self.data.get() },
        }
    }
}

impl PreemptLock<()> {
    pub unsafe fn unguarded_lock(&self) {
        cpu::preempt::disable();
    }

    pub unsafe fn unguarded_release(&self) {
        cpu::preempt::enable();
    }
}

impl<T: ?Sized + Default> Default for PreemptLock<T> {
    fn default() -> PreemptLock<T> {
        PreemptLock::new(Default::default())
    }
}

impl<'a, T: ?Sized> Deref for PreemptGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.data
    }
}

impl<'a, T: ?Sized> DerefMut for PreemptGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.data
    }
}

impl<'a, T: ?Sized> Drop for PreemptGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            cpu::preempt::enable();
        }
    }
}