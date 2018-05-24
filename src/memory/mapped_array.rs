// use core::ptr::NonNull;
// use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
// use core::ops::Deref;
// use core::{mem, slice};
// use nabi::{Result, Error};
// use arch::paging::PageMapper;
// use x86_64::VirtAddr;
// use x86_64::structures::paging::{Page, PageTableFlags};

// /// The mapped array starts off with
// /// a minimal amount of mapped physical memory.
// /// Over time, as it increases in size, it maps
// /// itself into a single array of virtual memory.
// pub struct MappedArray<T> {
//     /// The virtual start of the array.
//     ptr: NonNull<T>,
//     /// The max length of the array, in mem::size_of::<T>.
//     max_len: usize,
//     /// The current, mapped length of the array, in mem::size_of::<T>.
//     current_len: AtomicUsize,
// }

// impl<T> MappedArray<T> {
//     /// `max_size` is in bytes
//     pub const fn new(ptr: NonNull<T>, max_size: usize) -> MappedArray<T> {
//         MappedArray {
//             ptr,
//             max_len: max_size / mem::size_of::<T>(),
//             current_len: ATOMIC_USIZE_INIT,
//         }
//     }

//     /// Increase the mapped size by the specified size in bytes.
//     pub fn grow(&self, by: usize) -> Result<()> {
//         let current_len = self.current_len.load(Ordering::SeqCst);
//         let mut mapper = unsafe { PageMapper::new() };
//         let start_virt = VirtAddr::new(self.ptr.as_ptr() as u64 + current_len * mem::size_of::<T>() as u64);
//         let end_virt = start_virt + by;
//         let start_page = Page::containing_address(start_virt);
//         let end_page = Page::containing_address(end_virt);
//         let flags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
//         for page in Page::range(start_page, end_page) {
//             mapper.map(page, flags)
//                 .map_err(|_| Error::NO_MEMORY)?
//                 .flush();
//         }
//         self.current_len.store(current_len + by / mem::size_of::<T>(), Ordering::SeqCst);
//         Ok(())
//     }
// }

// impl<T> Deref for MappedArray<T> {
//     type Target = [T];
//     fn deref(&self) -> &[T] {
//         let current_len = self.current_len.load(Ordering::SeqCst);
//         unsafe {
//             slice::from_raw_parts(self.start.as_ptr(), current_len)
//         }
//     }
// }
