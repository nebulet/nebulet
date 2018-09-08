use nabi::{Error, Result};
use nebulet_derive::nebulet_abi;
use object::{Dispatcher, HandleRights, UserHandle};
use wasm::UserData;

#[nebulet_abi]
pub fn handle_close(handle: UserHandle<Dispatcher>, user_data: &UserData) -> Result<u32> {
    let mut handle_table = user_data.process.handle_table().write();
    handle_table.free_uncasted(handle)?;

    Ok(0)
}

#[nebulet_abi]
pub fn handle_duplicate(
    handle: UserHandle<Dispatcher>,
    new_rights: HandleRights,
    user_data: &UserData,
) -> Result<u32> {
    let mut handle_table = user_data.process.handle_table().write();

    handle_table
        .duplicate_uncasted(handle, new_rights)
        .map(|handle| handle.inner())
}
