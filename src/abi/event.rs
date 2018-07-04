use object::{EventDispatcher, HandleRights};
use nabi::{Result, Error};
use wasm::UserData;
use nebulet_derive::nebulet_abi;

#[nebulet_abi]
pub fn event_create(user_data: &UserData) -> Result<u32> {
    let mut handle_table = user_data.process.handle_table().write();

    let event = EventDispatcher::new();

    let flags = HandleRights::WRITE | HandleRights::READ | HandleRights::TRANSFER;

    handle_table
        .allocate(event, flags)
        .map(|handle| handle.inner())
}

// #[nebulet_abi]
// pub fn event_rearm(event_handle: UserHandle<EventDispatcher>, user_data: &UserData) -> Result<u32> {
//     let event = {
//         let handle_table = user_data.process.handle_table().read();

//         let handle = handle_table
//             .get(event_handle)?;
//         handle.check_rights(HandleRights::WRITE)?;
//         handle
//     };

//     match event.rearm() {
//         true => Ok(0),
//         false => Err(Error::BAD_STATE)
//     }
// }
