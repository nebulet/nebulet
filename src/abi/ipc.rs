use object::{Channel, Message, HandleRights, UserHandle};
use wasm::UserData;
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

#[nebulet_abi]
pub fn channel_create(handle_tx_offset: u32, handle_rx_offset: u32, user_data: &UserData) -> Result<u32> {
    let channel = Channel::new()?;
    
    let (handle_tx, handle_rx) = {
        let mut handle_table = user_data.process.handle_table().write();
        
        (
            handle_table.allocate(channel.clone(), HandleRights::all() ^ HandleRights::READ)?,
            handle_table.allocate(channel, HandleRights::all() ^ HandleRights::WRITE)?,
        )
    };

    {
        let instance = &user_data.instance;
        let mut memory = instance.memories[0].write();

        let h_tx = memory.carve_mut::<u32>(handle_tx_offset)?;
        *h_tx = handle_tx.inner();

        let h_rx = memory.carve_mut::<u32>(handle_rx_offset)?;
        *h_rx = handle_rx.inner();
    }

    Ok(0)
}

/// Write a message to the specified channel.
#[nebulet_abi]
pub fn channel_write(channel_handle: UserHandle<Channel>, buffer_offset: u32, buffer_size: u32, user_data: &UserData) -> Result<u32> {
    let msg = {
        let instance = &user_data.instance;
        let wasm_memory = &instance.memories[0].read();
        let data = wasm_memory.carve_slice(buffer_offset, buffer_size)
            .ok_or(Error::INVALID_ARG)?;
        Message::new(data, vec![])
    };
    
    let handle_table = user_data.process.handle_table().read();

    handle_table
        .get(channel_handle)?
        .check_rights(HandleRights::WRITE)?
        .write(msg)?;

    Ok(0)
}

/// Read a message from the specified channel.
#[nebulet_abi]
pub fn channel_read(channel_handle: UserHandle<Channel>, buffer_offset: u32, buffer_size: u32, msg_size_out: u32, user_data: &UserData) -> Result<u32> {
    let chan = {
        let handle_table = user_data.process.handle_table().read();

        let handle = handle_table
            .get(channel_handle)?;
        
        handle.check_rights(HandleRights::READ)?;

        handle
    };

    let msg = chan.read()?;

    let instance = &user_data.instance;
    let mut memory = instance.memories[0].write();

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
