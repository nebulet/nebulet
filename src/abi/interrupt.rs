use nabi::{Result, Error};
use object::{UserHandle, HandleRights, Channel};
use object::interrupt::{Interrupt, InterruptFlags};
use wasm::UserData;
use nebulet_derive::nebulet_abi;

#[nebulet_abi]
pub fn interrupt_create(channel_handle: UserHandle<Channel>, vector: u32, user_data: &UserData) -> Result<u32> {
    let channel = {
        let handle_table = user_data.process.handle_table().read();

        let handle = handle_table
            .get(channel_handle)?;
        
        handle.check_rights(HandleRights::WRITE)?;

        handle.dispatcher().copy_ref()
    };

    let interrupt = Interrupt::new(channel, InterruptFlags::UNMASK_PREWAIT | InterruptFlags::MASK_POSTWAIT, vector);

    interrupt.register()?;

    {
        let mut handle_table = user_data.process.handle_table().write();
        let flags = HandleRights::WRITE | HandleRights::READ;

        handle_table
            .allocate(interrupt, flags)
            .map(|handle| handle.inner())
    }
}

#[nebulet_abi]
pub fn interrupt_ack(interrupt_handle: UserHandle<Interrupt>, user_data: &UserData) -> Result<u32> {
    let handle_table = user_data.process.handle_table().read();

    let handle = handle_table
        .get(interrupt_handle)?;
    
    handle.check_rights(HandleRights::WRITE)?;
    
    handle.dispatcher().ack()?;

    Ok(0)
}