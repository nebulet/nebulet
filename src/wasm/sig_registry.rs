use cretonne_codegen::ir::Signature;
use alloc::Vec;
use spin::RwLock;

pub struct SigRegistry {
    table: RwLock<Vec<Signature>>,
}

impl SigRegistry {
    pub fn new() -> SigRegistry {
        SigRegistry {
            table: RwLock::new(Vec::new()),
        }
    }

    pub fn get_id(sig: &Signature)
}
