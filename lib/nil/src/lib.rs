//! `nil` stands for `Nebulet Internal Library`
//! and contains various types that assist in
//! writing and working on Nebulet.

#![no_std]
#![feature(
    repr_transparent,
    box_syntax,
    dropck_eyepatch,
    allocator_api,
    alloc,
    unsize,
    coerce_unsized,
    box_into_raw_non_null,
)]
#![deny(warnings)]

extern crate alloc;

mod refptr;

pub use refptr::{Ref, KernelRef};
