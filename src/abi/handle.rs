use object::{ProcessRef, HandleRights, HandleOffset};
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

#[nebulet_abi]
pub fn handle_close(handle: HandleOffset, process: &ProcessRef) -> Result<u32> {
    let mut handle_table = process.handle_table().write();
    handle_table.free(handle as usize)?;

    Ok(0)
}

#[nebulet_abi]
pub fn handle_duplicate(handle: HandleOffset, new_rights: HandleRights, process: &ProcessRef) -> Result<u32> {
    let mut handle_table = process.handle_table().write();

    handle_table.duplicate(handle as usize, new_rights)
        .map(|handle| handle as u32)
}
