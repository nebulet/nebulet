use object::{HandleRights, UserHandle};
use wasm::UserData;
use nabi::{Result, Error};
use nil::HandleRef;
use nebulet_derive::nebulet_abi;

#[nebulet_abi]
pub fn handle_close(handle: UserHandle<HandleRef>, user_data: &UserData) -> Result<u32> {
    let mut handle_table = user_data.process.handle_table().write();
    handle_table.free_uncasted(handle)?;

    Ok(0)
}

#[nebulet_abi]
pub fn handle_duplicate(handle: UserHandle<HandleRef>, new_rights: HandleRights, user_data: &UserData) -> Result<u32> {
    let mut handle_table = user_data.process.handle_table().write();

    handle_table.duplicate_uncasted(handle, new_rights)
        .map(|handle| handle.inner())
}
