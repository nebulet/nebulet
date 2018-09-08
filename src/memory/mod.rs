//! Lets allocate some memory for SIPs, guys and gals

mod region;
pub mod sip;
// pub mod mapped_array;

pub use self::region::{LazyRegion, MemFlags, Region};
pub use self::sip::{WasmMemory, WasmStack};
// pub use self::mapped_array::MappedArray;

use arch::lock::Spinlock;
// use core::ptr::NonNull;
// use object::Handle;

static SIP_ALLOCATOR: Spinlock<sip::SipAllocator> = Spinlock::new(sip::SipAllocator::new(
    ::SIP_MEM_OFFSET,
    ::SIP_MEM_OFFSET + ::SIP_MEM_SIZE,
));

// pub static HANDLE_TABLE: MappedArray<Handle> = MappedArray::new(
//     unsafe { NonNull::new_unchecked(::HANDLE_TABLE_OFFSET as _) },
//     ::HANDLE_TABLE_SIZE
// );
