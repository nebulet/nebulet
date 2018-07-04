use super::dispatcher::{Dispatcher, StateObserver, ObserverResult};
use object::Handle;
use event::Event;
use signals::Signal;
use alloc::arc::Arc;

pub struct WaitObserver {
    watched_signals: Signal,
    wakeup_reasons: Signal,
    event: Arc<Event>,
}

impl WaitObserver {
    pub fn new(event: Arc<Event>, watched_signals: Signal) -> WaitObserver {
        WaitObserver {
            watched_signals,
            wakeup_reasons: Signal::empty(),
            event,
        }
    }

    pub fn finalize(self) -> Signal {
        self.wakeup_reasons
    }
}

impl StateObserver for WaitObserver {
    fn on_init(&mut self, initial_state: Signal) -> ObserverResult {
        self.wakeup_reasons = initial_state;

        if self.watched_signals.intersects(initial_state) {
            self.event.signal(false);
        }

        ObserverResult::Keep
    }

    fn on_state_change(&mut self, new_state: Signal) -> ObserverResult {
        self.wakeup_reasons |= new_state;

        if self.wakeup_reasons.intersects(new_state) {
            self.event.signal(false);
        }

        ObserverResult::Keep
    }

    fn on_destruction(&mut self, _handle: &Handle<Dispatcher>) -> ObserverResult {
        ObserverResult::Keep
    }

    fn on_removal(&mut self) {
        self.event.signal(false);
    }
}
