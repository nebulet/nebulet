use core::ptr::NonNull;
use core::mem;

/// A sorted tree of branches
pub struct TreeList {
    top: NonNull<Branch>,
}

impl TreeList {
    /// Create an empty tree
    pub const fn empty() -> TreeList {
        TreeList {
            // Random non-real pointer
            top: NonNull::new_unchecked(0x1 as *mut _),
        }
    }

    /// Creates a `TreeList` that contains the given memory.
    /// Actually writes to that area, so unsafe
    pub unsafe fn new(addr: *mut u8, size: usize) -> TreeList {
        debug_assert!(size >= Self::min_size());
        debug_assert!(!addr.is_null());

        let ptr = addr as *mut Branch;

        let branch = Branch {
            size: size,
            prev_addr: None,
            next_addr: None,
            left: None,
            right: None,
        };

        ptr.write(branch);

        TreeList {
            top: NonNull::new_unchecked(ptr),
        }
    }

    pub fn min_size() -> usize {
        mem::size_of::<Branch>()
    }
}

struct BranchInfo {
    addr: *const Branch,
    size: usize,
}

pub struct Branch {
    // The size of the memory refered to by this branch
    size: usize,
    // The previous branch by addr
    prev_addr: Option<NonNull<Branch>>,
    // The next branch by addr
    next_addr: Option<NonNull<Branch>>,

    left: Option<NonNull<Branch>>,
    right: Option<NonNull<Branch>>,
}

impl Branch {
    /// Returns info about this branch
    fn info(&self) -> BranchInfo {
        BranchInfo {
            addr: self as *const Branch,
            size: self.size,
        }
    }
}