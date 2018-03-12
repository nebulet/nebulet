//! A `Compilation` contains the compiled function bodies for a WebAssembly
//! module

use super::module::Module;

use alloc::Vec;

#[derive(Debug)]
pub struct Compilation<'module> {
    /// The module this is instantiated from
    pub module: &'module Module,

    /// Compiled machine code for the function bodies
    pub functions: Vec<Vec<u8>>,
}

impl<'module> Compilation<'module> {
    /// Allocates the runtime data structures with the given flags
    pub fn new(module: &'module Module, functions: Vec<Vec<u8>>) -> Compilation {
        Compilation {
            module,
            functions,
        }
    }
}