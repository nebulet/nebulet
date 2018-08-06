use object::{Channel, HandleRights, UserHandle};
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;
use wasm::UserData;

#[nebulet_abi]
pub fn service_create(name_offset: u32, name_len: u32, channel_handle: UserHandle<Channel>, user_data: &UserData) -> 