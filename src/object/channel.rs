use nil::{Ref, KernelRef};
use nabi::{Result};
use object::Handle;
use alloc::Vec;
use spin::RwLock;

/// Represents a writable
/// and readable channel
/// for transferring data
/// between processes.
#[derive(KernelRef)]
pub struct ChannelRef {
    data_buffer: RwLock<Vec<u8>>,
    handle_buffer: RwLock<Vec<Handle>>,
}

impl ChannelRef {
    pub fn new() -> Result<Ref<Self>> {
        Ok(ChannelRef {
            data_buffer: RwLock::new(Vec::new()),
            handle_buffer: RwLock::new(Vec::new()),
        }.into())
    }

    // pub fn write_data(&self, data: &[u8]) -> Result<()> {

    // }
}
