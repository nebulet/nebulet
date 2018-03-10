//! The interface between running processes and the kernel
//! 

// use context;
use macros::println;

use nabi::{Result, Error};

pub macro define_abi {
    ($name:ident, |$arg0:ident: $arg0_type:ty| $code:block) => {
        pub extern "C" fn $name (abi_arg0: $arg0_type) -> usize {
            // The inner function
            fn inner($arg0: $arg0_type) -> Result<usize> {
                $code
            }
            // convert the result to a usize
            let result = inner(abi_arg0);
            Error::mux(result)
        }
    }
}

define_abi!(vga_test, |s: &str| {
    println!("{}", s);

    Ok(0)
});