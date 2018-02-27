use alloc::heap::{Alloc, AllocErr, Layout};
use linked_list_allocator::Heap;
use spin::Mutex;

use paging::ActivePageTable;

static HEAP: Mutex<Option<Heap>> = Mutex::new(None);

pub struct Allocator;

impl Allocator {
    pub unsafe fn init(offset: usize, size: usize) {
        *HEAP.lock() = Some(Heap::new(offset, size));
    }
}

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, mut layout: Layout) -> Result<*mut u8, AllocErr> {
        loop {
            let res = if let Some(ref mut heap) = *HEAP.lock() {
                heap.allocate_first_fit(layout)
            } else {
                panic!("HEAP not initialized");
            };

            match res {
                Err(AllocErr::Exhausted { request }) => {
                    layout = request;

                    let size = if let Some(ref heap) = *HEAP.lock() {
                        heap.size()
                    } else {
                        panic!("HEAP not initialized");
                    };

                    super::map_heap(&mut ActivePageTable::new(), ::KERNEL_HEAP_OFFSET + size, ::KERNEL_HEAP_SIZE);

                    if let Some(ref mut heap) = *HEAP.lock() {
                        heap.extend(::KERNEL_HEAP_SIZE);
                    } else {
                        panic!("HEAP not initialized");
                    }
                },
                other => return other,
            }
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.deallocate(ptr, layout)
        } else {
            panic!("HEAP not initialized");
        }
    }

    fn oom(&mut self, error: AllocErr) -> ! {
        panic!("Out of memory: {:?}", error);
    }

    fn usable_size(&self, layout: &Layout) -> (usize, usize) {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.usable_size(layout)
        } else {
            panic!("HEAP not initialized");
        }
    }
}