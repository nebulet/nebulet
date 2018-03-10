/// This file contains various constants

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
pub const KERNEL_HEAP_SIZE: usize = 1 * 1024 * 1024; // 1MB


/// Size constants
pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;