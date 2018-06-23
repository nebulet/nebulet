use sync::atomic::{Atomic, Ordering};
use core::cell::Cell;
use core::ptr;
use alloc::boxed::Box;

unsafe impl<T: Sized> Sync for Mpsc<T> {}
unsafe impl<T: Sized> Send for Mpsc<T> {}

struct MpscNode<T: Sized> {
    item: T,
    next: *mut MpscNode<T>,
}

pub struct Mpsc<T: Sized> {
    pushlist: Atomic<*mut MpscNode<T>>,
    poplist: Cell<*mut MpscNode<T>>,
}

impl<T: Sized> Mpsc<T> {
    pub const fn new() -> Mpsc<T> {
        Mpsc {
            pushlist: Atomic::new(ptr::null_mut()),
            poplist: Cell::new(ptr::null_mut()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.poplist.get().is_null() && self.pushlist.load(Ordering::Relaxed).is_null()
    }

    pub fn get_sender(&self) -> Sender<T> {
        Sender {
            mpsc: self,
        }
    }

    #[inline]
    pub fn push(&self, item: T) {
        let old_head = self.pushlist.load(Ordering::Relaxed);

        let node = box MpscNode {
            item,
            next: old_head,
        };

        let node_ptr = Box::into_raw(node);

        while self.pushlist.compare_exchange_weak(old_head, node_ptr, Ordering::Release, Ordering::Relaxed).is_err() {}
    }

    #[inline]
    pub unsafe fn pop(&self) -> Option<T> {
        if !self.poplist.get().is_null() {
            // The poplist is not empty, so pop from it
            let node = self.poplist.get();
            self.poplist.set((*node).next);
            let boxed = Box::from_raw(node);
            Some(boxed.item)
        } else {
            // The poplist is empty, so atomically take
            // the entire pushlist and reverse it into the poplist
            let mut node = self.pushlist.swap(ptr::null_mut(), Ordering::Acquire);
            if node.is_null() {
                return None;
            }

            while !(*node).next.is_null() {
                let next = (*node).next;
                (*node).next = self.poplist.get();
                self.poplist.set(node);
                node = next;
            }

            let boxed = Box::from_raw(node);
            Some(boxed.item)
        }
    }
}

impl<T: Sized> Drop for Mpsc<T> {
    fn drop(&mut self) {
        while let Some(item) = unsafe { self.pop() } {
            drop(item);
        }
    }
}

#[derive(Clone)]
pub struct Sender<'a, T: 'a + Sized> {
    mpsc: &'a Mpsc<T>,
}

impl<'a, T: Sized> Sender<'a, T> {
    pub fn send(&self, item: T) {
        self.mpsc.push(item);
    }
}
