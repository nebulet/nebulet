#![allow(unused_macros)]

macro_rules! likely {
    ($e:expr) => {
        unsafe {
            ::core::intrinsics::likely($e)
        }
    };
}

macro_rules! unlikely {
    ($e:expr) => {
        unsafe {
            ::core::intrinsics::unlikely($e)
        }
    };
}

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        #[allow(unused_unsafe)]
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    }
}
