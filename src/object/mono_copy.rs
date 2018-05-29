use object::ProcessRef;
use nil::{Ref, KernelRef};
use nabi::{Result, Error};

#[derive(KernelRef)]
pub struct MonoCopyRef {
    /// The process that contains the buffer.
    process: Ref<ProcessRef>,
    buffer: (u32, u32),
}

impl MonoCopyRef {
    pub fn new(process: Ref<ProcessRef>, buffer: (u32, u32)) -> Result<Ref<MonoCopyRef>> {
        Ref::new(MonoCopyRef {
            process,
            buffer,
        })
    }

    /// Write data directly into the buffer contained in the process.
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        if data.len() <= self.buffer.1 as usize {
            let mut instance = self.process.instance().write();
            let memory = &mut instance.memories[0];
            let buffer = memory.carve_slice_mut(self.buffer.0, self.buffer.1)
                .ok_or(Error::OUT_OF_BOUNDS)?;
            // copy over the data
            buffer[0..data.len()].copy_from_slice(data);

            Ok(())
        } else {
            Err(Error::OUT_OF_BOUNDS)
        }
    }
}
