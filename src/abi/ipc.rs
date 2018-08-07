use object::{Channel, Stream, Message, HandleRights, UserHandle};
use wasm::UserData;
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

#[nebulet_abi]
pub fn channel_create(handle_tx_offset: u32, handle_rx_offset: u32, user_data: &UserData) -> Result<u32> {
    let (tx, rx) = Channel::new_pair();
    
    let (handle_tx, handle_rx) = {
        let mut handle_table = user_data.process.handle_table().write();
        
        (
            handle_table.allocate(tx, HandleRights::all() ^ HandleRights::READ ^ HandleRights::DUPLICATE)?,
            handle_table.allocate(rx, HandleRights::all() ^ HandleRights::WRITE)?,
        )
    };

    {
        let instance = &user_data.instance;
        let mut memory = &instance.memories[0];

        let h_tx = memory.carve_mut::<u32>(handle_tx_offset)?;
        *h_tx = handle_tx.inner();

        let h_rx = memory.carve_mut::<u32>(handle_rx_offset)?;
        *h_rx = handle_rx.inner();
    }

    Ok(0)
}

/// Write a message to the specified channel.
#[nebulet_abi]
pub fn channel_send(channel_handle: UserHandle<Channel>, buffer_offset: u32, buffer_size: u32, user_data: &UserData) -> Result<u32> {
    let msg = {
        let instance = &user_data.instance;
        let wasm_memory = &instance.memories[0];
        let data = wasm_memory.carve_slice(buffer_offset, buffer_size)
            .ok_or(Error::INVALID_ARG)?;
        Message::new(data, vec![])?
    };
    
    let handle_table = user_data.process.handle_table().read();

    handle_table
        .get(channel_handle)?
        .check_rights(HandleRights::WRITE)?
        .send(msg)?;

    Ok(0)
}

/// Read a message from the specified channel.
#[nebulet_abi]
pub fn channel_recv(channel_handle: UserHandle<Channel>, buffer_offset: u32, buffer_size: u32, msg_size_out: u32, user_data: &UserData) -> Result<u32> {
    let chan = {
        let handle_table = user_data.process.handle_table().read();

        let handle = handle_table
            .get(channel_handle)?;
        
        handle.check_rights(HandleRights::READ)?;

        handle
    };

    let first_msg_len = chan.first_msg_len()?;

    let instance = &user_data.instance;
    let mut memory = &instance.memories[0];

    let msg_size = memory.carve_mut::<u32>(msg_size_out)?;
    *msg_size = first_msg_len as u32;
    
    if first_msg_len > buffer_size as usize {
        return Err(Error::BUFFER_TOO_SMALL);
    }

    let msg = chan.recv()?;

    let write_buf = memory.carve_slice_mut(buffer_offset, buffer_size)
        .ok_or(Error::INVALID_ARG)?;

    if write_buf.len() < msg.data().len() {
        Err(Error::BUFFER_TOO_SMALL)
    } else {
        write_buf.copy_from_slice(msg.data());

        Ok(0)
    }
}

#[nebulet_abi]
pub fn stream_create(handle_tx_offset: u32, handle_rx_offset: u32, user_data: &UserData) -> Result<u32> {
    let (tx, rx) = Stream::new_pair();
    
    let (handle_tx, handle_rx) = {
        let mut handle_table = user_data.process.handle_table().write();
        
        (
            handle_table.allocate(tx, HandleRights::all() ^ HandleRights::READ ^ HandleRights::DUPLICATE)?,
            handle_table.allocate(rx, HandleRights::all() ^ HandleRights::WRITE)?,
        )
    };

    {
        let instance = &user_data.instance;
        let mut memory = &instance.memories[0];

        let h_tx = memory.carve_mut::<u32>(handle_tx_offset)?;
        *h_tx = handle_tx.inner();

        let h_rx = memory.carve_mut::<u32>(handle_rx_offset)?;
        *h_rx = handle_rx.inner();
    }

    Ok(0)
}

#[nebulet_abi]
pub fn stream_write(stream_handle: UserHandle<Stream>, buffer_offset: u32, buffer_size: u32, written_size_out: u32, user_data: &UserData) -> Result<u32> {
    let instance = &user_data.instance;
    let mut memory = &instance.memories[0];
    let data = memory.carve_slice(buffer_offset, buffer_size)
        .ok_or(Error::INVALID_ARG)?;
    
    let handle_table = user_data.process.handle_table().read();

    let stream = handle_table.get(stream_handle)?;

    stream.check_rights(HandleRights::WRITE)?;

    let written_len = stream.write(data)?;

    let written_out = memory.carve_mut::<u32>(written_size_out)?;
    *written_out = written_len as u32;
    
    Ok(0)
}

#[nebulet_abi]
pub fn stream_read(stream_handle: UserHandle<Stream>, buffer_offset: u32, buffer_size: u32, read_size_out: u32, user_data: &UserData) -> Result<u32> {
    let handle_table = user_data.process.handle_table().read();

    let stream = handle_table.get(stream_handle)?;

    stream.check_rights(HandleRights::READ)?;

    let instance = &user_data.instance;
    let mut memory = &instance.memories[0];

    let mut data = memory.carve_slice_mut(buffer_offset, buffer_size)
        .ok_or(Error::INVALID_ARG)?;

    let read_size = stream.read(&mut data)?;

    let out = memory.carve_mut::<u32>(read_size_out)?;
    *out = read_size as u32;

    Ok(0)
}