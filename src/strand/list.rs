use alloc::LinkedList;
use alloc::arc::Arc;

use arch::lock::Spinlock;
use strand::Strand;

/// Per-cpu strand list
pub struct StrandList {
    /// Linked list of strands.
    strands: LinkedList<Arc<Spinlock<Strand>>>,
}

impl StrandList {
    /// Creates a new, empty list.
    pub fn new() -> StrandList {
        let mut list = LinkedList::new();

        // initial idle strand
        // let mut idle_strand = Strand::new("[idle]", entry)
        //     .expect("Could not create the idle strand");

        StrandList {
            strands: list,
        }
    }
}

extern fn idle_strand_entry(_: usize) -> i32 {
    use arch::interrupt::halt;
    loop {
        unsafe { halt(); }
    }
    0
}
