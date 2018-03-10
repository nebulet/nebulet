use alloc::Vec;
use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use core::mem;

enum Slot<T> {
    /// Vacant slot, containing index to the next vacant slot
    Vacant(AtomicUsize),
    /// Occupied slot, containing value
    Occupied(T),
}

pub struct Arena<T> {
    /// Object slots
    slots: Vec<Slot<T>>,
    /// number of occupied slots
    len: AtomicUsize,
    /// Index of first vacant slot
    head: AtomicUsize,
}

impl<T> Arena<T> {
    /// Create an Arena with space for `capacity` objects.
    pub fn new(capacity: usize) -> Arena<T> {
        Arena {
            slots: Vec::with_capacity(size),
            len: ATOMIC_USIZE_INIT,
            head: !ATOMIC_USIZE_INIT,
        }
    }

    pub fn alloc(&mut self, object: T) -> Option<&mut T> {
        let old_len = self.len.fetch_add(1, Ordering::Relaxed);

        let head = self.head.load(Ordering::Relaxed);
        
        if head == !0 {
            self.slots.push(Slot::Occupied(object));
            old_len
        } else {
            let index = head;
            match self.slots[index] {
                Slot::Vacant(next) => {
                    self.head.store(next, Ordering::Relaxed);
                    self.slots[index] = Slot::Occupied(object);
                },
                Slot::Occupied(_) => unreachable!(),
            };
            index
        }
    }
}