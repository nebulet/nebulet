use alloc::heap::{Alloc, AllocErr, Layout};
use linked_list_allocator::Heap;

use arch::lock::PreemptLock;
use arch::paging::PageMapper;

static HEAP: PreemptLock<Heap> = PreemptLock::new(Heap::empty());

pub struct Allocator;

impl Allocator {
    pub unsafe fn init(offset: usize, size: usize) {
        HEAP.lock().init(offset, size);
    }
}

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, mut layout: Layout) -> Result<*mut u8, AllocErr> {
        loop {
            let res = HEAP.lock().allocate_first_fit(layout);

            match res {
                Err(AllocErr::Exhausted { request }) => {
                    layout = request;
                    
                    let size = HEAP.lock().size();

                    super::map_heap(&mut PageMapper::new(), ::KERNEL_HEAP_OFFSET + size, ::KERNEL_HEAP_SIZE);

                    HEAP.lock().extend(::KERNEL_HEAP_SIZE);
                },
                other => return other,
            }
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        HEAP.lock().deallocate(ptr, layout);
    }

    fn oom(&mut self, error: AllocErr) -> ! {
        panic!("Out of memory: {:?}", error);
    }

    fn usable_size(&self, layout: &Layout) -> (usize, usize) {
        HEAP.lock().usable_size(layout)
    }
}