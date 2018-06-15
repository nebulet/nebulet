use core::sync::atomic::{AtomicPtr, Ordering};
use core::cell::Cell;
use core::ptr;
use alloc::boxed::Box;
use alloc::arc::Arc;

struct MpscNode<T: Sized> {
    item: T,
    next: *mut MpscNode<T>,
}

pub struct Mpsc<T: Sized> {
    pushlist: AtomicPtr<MpscNode<T>>,
    poplist: Cell<*mut MpscNode<T>>,
}

impl<T: Sized> Mpsc<T> {
    pub fn new() -> (Sender<T>, Reciever<T>) {
        let mpsc = Arc::new(Mpsc {
            pushlist: AtomicPtr::new(ptr::null_mut()),
            poplist: Cell::new(ptr::null_mut()),
        });

        let tx = Sender { mpsc: Arc::clone(&mpsc) };
        let rx = Reciever { mpsc };

        (tx, rx)
    }

    fn push(&self, item: T) {
        let old_head = self.pushlist.load(Ordering::Relaxed);

        let node = Box::new(MpscNode {
            item,
            next: old_head,
        });

        let node_ptr = Box::into_raw(node);

        while self.pushlist.compare_exchange_weak(old_head, node_ptr, Ordering::Release, Ordering::Relaxed).is_err() {}
    }

    fn pop(&self) -> Option<T> {
        if !self.poplist.get().is_null() {
            // The poplist is not empty, so pop from it
            let node_ptr = self.poplist.get();
            let node_ref = unsafe { &*node_ptr };
            self.poplist.set(node_ref.next);
            let boxed = unsafe { Box::from_raw(node_ptr) };
            Some(boxed.item)
        } else {
            // The poplist is empty, so atomically take
            // the entire pushlist and reverse it into the poplist
            let node_ptr = self.pushlist.swap(ptr::null_mut(), Ordering::Acquire);
            if node_ptr.is_null() {
                return None; // both the pushlist and poplist were empty
            }

            let mut node_ref = unsafe { &mut *node_ptr };

            while !node_ref.next.is_null() {
                let next_ptr = node_ref.next;
                node_ref.next = self.poplist.get();
                self.poplist.set(node_ref);
                node_ref = unsafe { &mut *next_ptr };
            }

            let boxed = unsafe { Box::from_raw(node_ref) };
            Some(boxed.item)
        }
    }
}

pub struct Reciever<T: Sized> {
    mpsc: Arc<Mpsc<T>>,
}

impl<T: Sized> Reciever<T> {
    pub fn recv(&self) -> Option<T> {
        self.mpsc.pop()
    }
}

#[derive(Clone)]
pub struct Sender<T: Sized> {
    mpsc: Arc<Mpsc<T>>,
}

impl<T: Sized> Sender<T> {
    pub fn send(&self, item: T) {
        self.mpsc.push(item);
    }
}
