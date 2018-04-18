use alloc::LinkedList;
use alloc::arc::{Arc, Weak};

use arch::lock::Spinlock;
use strand::Strand;

/// Per-cpu strand list
pub struct StrandList {
    /// Linked list of strands.
    full_list: LinkedList<Arc<Spinlock<Strand>>>,
    ready_list: LinkedList<Weak<Spinlock<Strand>>>,
    idle_strand: Arc<Spinlock<Strand>>,
}

impl StrandList {
    /// Creates a new, empty list.
    pub fn new() -> StrandList {
        let full_list = LinkedList::new();
        let ready_list = LinkedList::new();

        // initial idle strand
        let idle_strand = Arc::new(Spinlock::new(Strand::new("[idle]", idle_strand_entry)
            .expect("Could not create the idle strand")));

        StrandList {
            full_list,
            ready_list,
            idle_strand,
        }
    }

    pub fn pop_ready(&mut self) -> Weak<Spinlock<Strand>> {
        if let Some(weak_strand) = self.ready_list.pop_front() {
            weak_strand
        } else {
            Arc::downgrade(&self.idle_strand)
        }
    }
}

extern fn idle_strand_entry()  {
    use arch::interrupt::halt;
    loop {
        unsafe { halt(); }
    }
}
