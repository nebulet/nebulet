use alloc::heap::{Alloc, AllocErr, Layout};
use spin::Mutex;

use arch::paging::ActivePageTable;

struct BumpHeap {
    start: usize,
    size: usize,
    next: usize,
}

impl BumpHeap {
    pub const fn empty() -> BumpHeap {
        BumpHeap {
            start: 0,
            size: 0,
            next: 0,
        }
    }

    pub unsafe fn init(&mut self, offset: usize, size: usize) {
        self.start = offset;
        self.size = size;
        self.next = offset;
    }

    pub unsafe fn extend(&mut self, by: usize) {
        self.size += by;
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let alloc_start = align_up(self.next, layout.align());
        let alloc_end = alloc_start.saturating_add(layout.size());

        if alloc_end <= self.start + self.size {
            self.next = alloc_end;
            Ok(alloc_start as *mut u8)
        } else {
            Err(AllocErr::Exhausted { request: layout })
        }
    }
}

static HEAP: Mutex<BumpHeap> = Mutex::new(BumpHeap::empty());

pub struct Allocator;

impl Allocator {
    pub unsafe fn init(offset: usize, size: usize) {
        HEAP.lock().init(offset, size);
    }
}

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, mut layout: Layout) -> Result<*mut u8, AllocErr> {
        loop {
            let res = HEAP.lock().alloc(layout);

            match res {
                Err(AllocErr::Exhausted { request }) => {
                    println!("Expanding heap");
                    layout = request;

                    let size = HEAP.lock().size;

                    super::map_heap(&mut ActivePageTable::new(), ::KERNEL_HEAP_OFFSET + size, ::KERNEL_HEAP_SIZE);

                    HEAP.lock().extend(::KERNEL_HEAP_SIZE);
                },
                other => {
                    return other;
                },
            }
        }
    }

    unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        // HEAP.lock().deallocate(ptr, layout);
    }

    fn oom(&mut self, error: AllocErr) -> ! {
        panic!("Out of memory: {:?}", error);
    }

    // fn usable_size(&self, layout: &Layout) -> (usize, usize) {
    //     HEAP.lock().usable_size(layout)
    // }
}

/// Align downwards. Returns the greatest x with alignment `align`
/// so that x <= addr. The alignment must be a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

/// Align upwards. Returns the smallest x with alignment `align`
/// so that x >= addr. The alignment must be a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}