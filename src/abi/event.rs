use object::{Process, Event, HandleRights, UserHandle};
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;
use nil::Ref;

#[nebulet_abi]
pub fn event_create(process: &Process) -> Result<u32> {
    let mut handle_table = process.handle_table().write();

    let event = Ref::new(Event::new())?;

    let flags = HandleRights::WRITE | HandleRights::READ | HandleRights::TRANSFER;

    handle_table
        .allocate(event, flags)
        .map(|handle| handle.inner())
}

#[nebulet_abi]
pub fn event_wait(event_handle: UserHandle<Event>, process: &Process) -> Result<u32> {
    let event = {
        let handle_table = process.handle_table().read();

        let handle = handle_table
            .get(event_handle)?;
        handle.check_rights(HandleRights::WRITE)?;
        handle
    };

    event.wait();

    Ok(0)
}

/// Returns `ACCESS_DENIED` when the thread that attempts to trigger the event
/// is not the thread that created the event.
#[nebulet_abi]
pub fn event_trigger(event_handle: UserHandle<Event>, process: &Process) -> Result<u32> {
    let event = {
        let handle_table = process.handle_table().read();

        let handle = handle_table
            .get(event_handle)?;
        handle.check_rights(HandleRights::WRITE)?;
        handle
    };

    event.trigger().map(|count| count as u32)
}
