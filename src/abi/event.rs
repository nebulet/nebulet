use object::{Event, HandleRights, UserHandle};
use nabi::{Result, Error};
use wasm::UserData;
use nebulet_derive::nebulet_abi;
use nil::Ref;

#[nebulet_abi]
pub fn event_create(user_data: &UserData) -> Result<u32> {
    let mut handle_table = user_data.process.handle_table().write();

    let event = Ref::new(Event::new())?;

    let flags = HandleRights::WRITE | HandleRights::READ | HandleRights::TRANSFER;

    handle_table
        .allocate(event, flags)
        .map(|handle| handle.inner())
}

#[nebulet_abi]
pub fn event_wait(event_handle: UserHandle<Event>, user_data: &UserData) -> Result<u32> {
    let event = {
        let handle_table = user_data.process.handle_table().read();

        let handle = handle_table
            .get(event_handle)?;
        handle.check_rights(HandleRights::WRITE)?;
        handle
    };

    event.wait();

    Ok(0)
}

/// Poll an event to determine if it's done. Returns `0` if not done yet,
/// `1` if done, and < 0 if an error occured.
#[nebulet_abi]
pub fn event_poll(event_handle: UserHandle<Event>, user_data: &UserData) -> Result<u32> {
    let event = {
        let handle_table = user_data.process.handle_table().read();

        let handle = handle_table
            .get(event_handle)?;
        handle.check_rights(HandleRights::READ)?;
        handle
    };

    event.poll().map(|state| state as u32)
}

/// Returns `ACCESS_DENIED` when the thread that attempts to trigger the event
/// is not the thread that created the event.
#[nebulet_abi]
pub fn event_trigger(event_handle: UserHandle<Event>, user_data: &UserData) -> Result<u32> {
    let event = {
        let handle_table = user_data.process.handle_table().read();

        let handle = handle_table
            .get(event_handle)?;
        handle.check_rights(HandleRights::WRITE)?;
        handle
    };

    event.trigger().map(|count| count as u32)
}

#[nebulet_abi]
pub fn event_rearm(event_handle: UserHandle<Event>, user_data: &UserData) -> Result<u32> {
    let event = {
        let handle_table = user_data.process.handle_table().read();

        let handle = handle_table
            .get(event_handle)?;
        handle.check_rights(HandleRights::WRITE)?;
        handle
    };

    match event.rearm() {
        true => Ok(0),
        false => Err(Error::BAD_STATE)
    }
}
