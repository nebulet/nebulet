use object::{ProcessRef, MonoCopyRef, HandleRights};
use nil::Ref;
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

/// Creates a mono copy ipc handle.
/// Another process can write to this buffer,
/// assuming they have the handle.
#[nebulet_abi]
pub fn ipc_monocopy_create(buffer_offset: u32, buffer_size: u32, process: &Ref<ProcessRef>) -> Result<u32> {
    {
        let instance = process.instance().read();
        let memory = &instance.memories[0];

        // Validate buffer constraints
        memory.get_array(buffer_offset, buffer_size)
            .ok_or(Error::INVALID_ARG)?;
    }

    let mono_copy_ref = MonoCopyRef::new(process.clone(), (buffer_offset, buffer_size))?;

    {
        let mut handle_table = process.handle_table().write();

        handle_table.allocate(mono_copy_ref, HandleRights::WRITE)
            .map(|handle| handle as u32)
    }
}
