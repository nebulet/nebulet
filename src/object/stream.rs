use super::dispatcher::{Dispatch, Dispatcher};
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use arch::lock::Spinlock;
use core::cmp::min;
use nabi::{Error, Result};
use signals::Signal;

pub const BUFFER_SIZE: usize = 64 * 1024; // 64 KiB

struct SharedData {
    data: VecDeque<u8>,
}

/// Represents a writable
/// and readable stream
/// for transferring data
/// between processes.
pub struct Stream {
    shared: Arc<Spinlock<SharedData>>,
    peer: Spinlock<Option<Dispatch<Stream>>>,
}

impl Stream {
    pub fn new_pair() -> (Dispatch<Self>, Dispatch<Self>) {
        let shared = Arc::new(Spinlock::new(SharedData {
            data: VecDeque::new(),
        }));

        let first = Dispatch::new(Self {
            shared: Arc::clone(&shared),
            peer: Spinlock::new(None),
        });

        let second = Dispatch::new(Self {
            shared: Arc::clone(&shared),
            peer: Spinlock::new(Some(first.copy_ref())),
        });

        *first.peer.lock() = Some(second.copy_ref());

        (first, second)
    }

    pub fn write(self: &Dispatch<Self>, data: &[u8]) -> Result<usize> {
        let mut shared = self.shared.lock();

        let peer_guard = self.peer.lock();

        if let Some(peer) = peer_guard.as_ref() {
            let len_to_write = min(BUFFER_SIZE - shared.data.len(), data.len());

            shared.data.extend(&data[..len_to_write]);

            if shared.data.len() == BUFFER_SIZE {
                self.signal(Signal::empty(), Signal::WRITABLE)?;
            }

            peer.signal(Signal::READABLE, Signal::empty())?;

            Ok(len_to_write)
        } else {
            Err(Error::PEER_CLOSED)
        }
    }

    pub fn read(self: &Dispatch<Self>, out: &mut [u8]) -> Result<usize> {
        let mut shared = self.shared.lock();

        let peer_guard = self.peer.lock();

        if let Some(peer) = peer_guard.as_ref() {
            let len_to_read = min(shared.data.len(), out.len());

            for (src, dest) in shared.data.drain(..len_to_read).zip(out) {
                *dest = src;
            }

            if shared.data.len() < BUFFER_SIZE {
                peer.signal(Signal::WRITABLE, Signal::empty())?;
            }

            if shared.data.len() == 0 {
                self.signal(Signal::empty(), Signal::READABLE)?;
            }

            Ok(len_to_read)
        } else {
            Err(Error::PEER_CLOSED)
        }
    }
}

impl Dispatcher for Stream {
    fn allowed_user_signals(&self) -> Signal {
        Signal::READABLE | Signal::WRITABLE | Signal::PEER_CLOSED | Signal::PEER_SIGNALED
    }

    fn allows_observers(&self) -> bool {
        true
    }
}
