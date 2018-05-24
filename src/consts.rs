/// This file contains various constants

/// Version
pub const VERSION: &'static str = concat!(
    env!("CARGO_PKG_VERSION_MAJOR"),
    ".",
    env!("CARGO_PKG_VERSION_MINOR"),
);

/// Memory mapping
/// Top entry of P4 (511) is reserved for recursive mapping
/// Second from top entry of P4 is reserved for the kernel

pub const PML4_SIZE: usize = 0x0000_0080_0000_0000;
pub const PML4_MASK: usize = 0x0000_ff80_0000_0000;

/// Offset of recursive mapping
pub const RECURSIVE_PAGE_OFFSET: usize = (-(PML4_SIZE as isize)) as usize;
pub const RECURSIVE_PAGE_PML4: usize = (RECURSIVE_PAGE_OFFSET & PML4_MASK) / PML4_SIZE;

/// Offset of kernel
/// TODO: Actually map the kernel to here
pub const KERNEL_OFFSET: usize = RECURSIVE_PAGE_OFFSET - PML4_SIZE;
pub const KERNEL_PML4: usize = (KERNEL_OFFSET & PML4_MASK) / PML4_SIZE;

/// Offset of kernel heap
pub const KERNEL_HEAP_OFFSET: usize = KERNEL_OFFSET - PML4_SIZE;
pub const KERNEL_HEAP_PML4: usize = (KERNEL_HEAP_OFFSET & PML4_MASK) / PML4_SIZE;
/// Size of kernel heap
pub const KERNEL_HEAP_SIZE: usize = 20 * 1024 * 1024; // 1MB

/// Offset of the SIP heaps
/// The SIP Heap allocator starts here
/// and bumps up.
/// 
/// This starts at 2 GiB.
// pub const SIP_MEM_OFFSET: usize = KERNEL_HEAP_OFFSET - PML4_SIZE;
pub const SIP_MEM_OFFSET: usize = 1 << 31;

pub const SIP_MEM_SIZE: usize = KERNEL_HEAP_OFFSET - SIP_MEM_OFFSET;

/// Offset of the Handle Table.
/// This starts at 1 GiB
/// and goes up to 2 Gib.
pub const HANDLE_TABLE_OFFSET: usize = 1 << 30;
pub const HANDLE_TABLE_SIZE: usize = 1 << 30;
