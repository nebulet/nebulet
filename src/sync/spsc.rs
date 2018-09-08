pub use super::mpsc::IntrusiveNode;
use core::ptr;

unsafe impl<T> Send for IntrusiveSpsc<T> where T: IntrusiveNode + Send {}

unsafe impl<T> Sync for IntrusiveSpsc<T> where T: IntrusiveNode + Sync {}

pub struct IntrusiveSpsc<T: IntrusiveNode> {
    pushlist: *mut T,
    poplist: *mut T,
}

impl<T: IntrusiveNode> IntrusiveSpsc<T> {
    pub const fn new() -> IntrusiveSpsc<T> {
        IntrusiveSpsc {
            pushlist: ptr::null_mut(),
            poplist: ptr::null_mut(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.poplist.is_null() && self.pushlist.is_null()
    }

    #[inline]
    pub unsafe fn push(&mut self, item_ptr: *mut T) {
        debug_assert!(!item_ptr.is_on_queue());

        item_ptr.set_next(self.pushlist);

        self.pushlist = item_ptr;
    }

    #[inline]
    pub unsafe fn pop(&mut self) -> Option<*mut T> {
        if !self.poplist.is_null() {
            // The poplist is not empty, so pop from it
            let intrusive_node = self.poplist;
            self.poplist = intrusive_node.get_next();
            intrusive_node.set_next(ptr::null_mut());
            Some(intrusive_node)
        } else {
            // The poplist is empty, so atomically take
            // the entire pushlist and reverse it into the poplist
            let mut intrusive_node = self.pushlist;
            self.pushlist = ptr::null_mut();
            if intrusive_node.is_null() {
                return None;
            }

            while !intrusive_node.get_next().is_null() {
                let next = intrusive_node.get_next();
                intrusive_node.set_next(self.poplist);
                self.poplist = intrusive_node;
                intrusive_node = next;
            }

            debug_assert!(!intrusive_node.is_on_queue());

            Some(intrusive_node)
        }
    }
}
