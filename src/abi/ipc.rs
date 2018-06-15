use object::{Process, MonoCopyRef, ChannelRef, Message, HandleRights, HandleOffset};
use nil::Ref;
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

/// Creates a mono copy ipc handle.
/// Another process can write to this buffer,
/// assuming they have the handle.
#[nebulet_abi]
pub fn monocopy_create(buffer_offset: u32, buffer_size: u32, process: &Ref<Process>) -> Result<u32> {
    {
        let instance = process.instance().read();
        let memory = &instance.memories[0];

        // Validate buffer constraints
        memory.carve_slice(buffer_offset, buffer_size)
            .ok_or(Error::INVALID_ARG)?;
    }

    let mono_copy_ref = MonoCopyRef::new(process.clone(), (buffer_offset, buffer_size))?;

    {
        let mut handle_table = process.handle_table().write();

        handle_table.allocate(mono_copy_ref, HandleRights::WRITE | HandleRights::TRANSFER)
            .map(|handle| handle as u32)
    }
}

#[nebulet_abi]
pub fn channel_create(handle_tx_offset: u32, handle_rx_offset: u32, process: &Process) -> Result<u32> {
    let channel = ChannelRef::new()?;
    
    let (handle_tx, handle_rx) = {
        let mut handle_table = process.handle_table().write();
        
        (
            handle_table.allocate(channel.clone(), HandleRights::all() ^ HandleRights::READ)
                .map(|handle| handle as u32)?,
            handle_table.allocate(channel, HandleRights::all() ^ HandleRights::WRITE)
                .map(|handle| handle as u32)?,
        )
    };

    {
        let mut instance = process.instance().write();
        let memory = &mut instance.memories[0];

        let h_tx = memory.carve_mut::<u32>(handle_tx_offset)?;
        *h_tx = handle_tx;

        let h_rx = memory.carve_mut::<u32>(handle_rx_offset)?;
        *h_rx = handle_rx;
    }

    Ok(0)
}

/// Write a message to the specified channel.
#[nebulet_abi]
pub fn channel_write(channel_handle: HandleOffset, buffer_offset: u32, buffer_size: u32, process: &Process) -> Result<u32> {
    let msg = {
        let instance = process.instance().read();
        let wasm_memory = &instance.memories[0];
        let data = wasm_memory.carve_slice(buffer_offset, buffer_size)
            .ok_or(Error::INVALID_ARG)?;
        Message::new(data, vec![])
    };
    
    let handle_table = process.handle_table().read();

    handle_table
        .get(channel_handle as _)?
        .check_rights(HandleRights::WRITE)?
        .cast_ref::<ChannelRef>()?
        .write(msg)?;

    Ok(0)
}

/// Read a message from the specified channel.
#[nebulet_abi]
pub fn channel_read(channel_handle: HandleOffset, buffer_offset: u32, buffer_size: u32, msg_size_out: u32, process: &Process) -> Result<u32> {
    let chan_ref = {
        let handle_table = process.handle_table().read();

        handle_table
            .get(channel_handle as _)?
            .check_rights(HandleRights::READ)?
            .cast::<ChannelRef>()?
    };

    let msg = chan_ref.read()?;

    let mut instance = process.instance().write();
    let memory = &mut instance.memories[0];

    let msg_size = memory.carve_mut::<u32>(msg_size_out)?;
    *msg_size = msg.data().len() as u32;

    let write_buf = memory.carve_slice_mut(buffer_offset, buffer_size)
        .ok_or(Error::INVALID_ARG)?;

    if write_buf.len() < msg.data().len() {
        Err(Error::BUFFER_TOO_SMALL)
    } else {
        write_buf.copy_from_slice(msg.data());

        Ok(0)
    }
}
