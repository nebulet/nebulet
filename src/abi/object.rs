use object::{Dispatcher, UserHandle};
use object::dispatcher::LocalObserver;
use object::wait_observer::WaitObserver;
use event::{Event, EventVariant};
use signals::Signal;
use alloc::arc::Arc;
use nabi::{Result, Error};
use wasm::UserData;
use nebulet_derive::nebulet_abi;

#[nebulet_abi]
pub fn object_wait_one(object_handle: UserHandle<Dispatcher>, signals: Signal, user_data: &UserData) -> Result<u32> {
    let mut object = {
        let handle_table = user_data.process.handle_table().read();

        let handle = handle_table
            .get_uncasted(object_handle)?
            .copy_ref();
        handle
    };

    if !object.allowed_user_signals().contains(signals) {
        return Err(Error::INVALID_ARG);
    }

    let event = Arc::new(Event::new(EventVariant::Normal));

    let mut waiter = WaitObserver::new(Arc::clone(&event), signals);

    let local_observer = if let Some(observer) = LocalObserver::new(&mut waiter, &mut object) {
        observer
    } else {
        return Ok(0);
    };

    event.wait();

    // drop the local observer so we can access the waiter again.
    drop(local_observer);

    let wakeup_reasons = waiter.finalize();

    Ok(wakeup_reasons.bits())
}

#[nebulet_abi]
pub fn object_signal(object_handle: UserHandle<Dispatcher>, assert_signals: Signal, deassert_signals: Signal, user_data: &UserData) -> Result<u32> {
    let object = {
        let handle_table = user_data.process.handle_table().read();

        let handle = handle_table
            .get_uncasted(object_handle)?
            .copy_ref();
        handle
    };

    if !object.allowed_user_signals().contains(assert_signals | deassert_signals) {
        return Err(Error::INVALID_ARG);
    }

    object.signal(assert_signals, deassert_signals)?;

    Ok(0)
}