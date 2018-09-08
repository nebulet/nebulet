use super::super::map_heap;
use arch::paging::PageMapper;
use core::ptr;

pub unsafe fn alloc(size: usize) -> (*mut u8, usize, u32) {
    static mut OFFSET: usize = 0;

    let (ptr, actual_size) = map_heap(&mut PageMapper::new(), ::KERNEL_HEAP_OFFSET + OFFSET, size);

    OFFSET += actual_size;

    (ptr, actual_size, 0)
}

pub unsafe fn remap(_ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool) -> *mut u8 {
    // TODO: I think this can be implemented near the end?
    ptr::null_mut()
}

pub unsafe fn free_part(_ptr: *mut u8, _oldsize: usize, _newsize: usize) -> bool {
    false
}

pub unsafe fn free(_ptr: *mut u8, _size: usize) -> bool {
    false
}

pub fn can_release_part(_flags: u32) -> bool {
    false
}

pub fn allocates_zeros() -> bool {
    true
}

pub fn page_size() -> usize {
    4096
}
