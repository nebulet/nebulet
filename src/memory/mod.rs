//! Lets allocate some memory for SIPs, guys and gals

mod region;
mod code;
pub mod sip;

pub use self::region::Region;
pub use self::code::Code;
pub use self::sip::{WasmMemory, WasmStack};

use arch::lock::Spinlock;

static SIP_ALLOCATOR: Spinlock<sip::SipAllocator> = Spinlock::new(
    sip::SipAllocator::new(::SIP_MEM_OFFSET, ::SIP_MEM_OFFSET + ::SIP_MEM_SIZE)
);