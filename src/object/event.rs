use super::dispatcher::{Dispatch, Dispatcher};
use signals::Signal;

pub struct EventDispatcher;

impl EventDispatcher {
    /// Create a new event.
    /// The returned event can only
    /// be triggered by the process
    /// that created it.
    pub fn new() -> Dispatch<EventDispatcher> {
        Dispatch::new(EventDispatcher)
    }
}

impl Dispatcher for EventDispatcher {
    fn allowed_user_signals(&self) -> Signal {
        Signal::USER_ALL | Signal::EVENT_SIGNALED
    }

    fn allows_observers(&self) -> bool {
        true
    }
}
