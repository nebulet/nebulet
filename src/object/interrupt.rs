use sync::atomic::{Atomic, Ordering};
use object::dispatcher::{Dispatch, Dispatcher};
use object::channel::{Channel, Message};
use alloc::Vec;
use time::Instant;
use core::ops::Deref;
use core::{slice, mem};
use arch::interrupt;
use nabi::{Result, Error};

bitflags! {
    pub struct InterruptFlags: u32 {
        const UNMASK_PREWAIT = 1 << 0;
        const MASK_POSTWAIT = 1 << 1;
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum InterruptState {
    // Waiting,
    // Triggered,
    // Destroyed,
    NeedAck,
    Idle,
}

#[repr(C)]
struct InterruptPacket {
    seconds: u64,
    nanos: u32,
}

impl Deref for InterruptPacket {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const _ as *const u8, mem::size_of::<InterruptPacket>()) }
    }
}

pub struct Interrupt {
    channel: Dispatch<Channel>,
    state: Atomic<InterruptState>,
    flags: InterruptFlags,
    vector: u32,
}

impl Interrupt {
    pub fn new(channel: Dispatch<Channel>, flags: InterruptFlags, vector: u32) -> Dispatch<Interrupt> {
        Dispatch::new(Interrupt {
            channel,
            state: Atomic::new(InterruptState::Idle),
            flags,
            vector,
        })
    }

    fn send_packet(&self, timestamp: Instant) -> Result<()> {
        let duration = timestamp.duration_since(Instant::EPOCH);
        let packet = InterruptPacket {
            seconds: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        };

        let msg = Message::new(&packet, Vec::new())?;
        self.channel.send(msg)
    }

    pub fn mask(&self) {
        unsafe {
            interrupt::mask(self.vector as _);
        }
    }

    pub fn unmask(&self) {
        unsafe {
            interrupt::unmask(self.vector as _);
        }
    }

    fn interrupt_handler(this: *const ()) {
        let this = unsafe { &*(this as *const Self) };

        if this.flags.contains(InterruptFlags::MASK_POSTWAIT) {
            this.mask();
        }

        this.handle();
    }

    fn handle(&self) {
        if self.state.load(Ordering::Relaxed) == InterruptState::NeedAck {
            return;
        }

        let now = Instant::now();
        // ignore result
        let _ = self.send_packet(now);

        self.state.store(InterruptState::NeedAck, Ordering::Relaxed);
    }

    pub fn ack(&self) -> Result<()> {
        let state = self.state.load(Ordering::Relaxed);

        match state {
            InterruptState::NeedAck => {
                self.state.store(InterruptState::Idle, Ordering::Relaxed);
                if self.flags.contains(InterruptFlags::UNMASK_PREWAIT) {
                    self.unmask();
                }
                Ok(())
            },
            _ => unimplemented!(),
        }
    }

    pub fn register(&self) -> Result<()> {
        if unsafe { interrupt::register_handler(self.vector, Self::interrupt_handler, self as *const _ as *const _) } {
            Ok(())
        } else {
            Err(Error::INVALID_ARG)
        }
    }

    pub fn unregister(&self) -> Result<()> {
        if unsafe { interrupt::unregister_handler(self.vector) } {
            Ok(())
        } else {
            Err(Error::INVALID_ARG)
        }
    }
}

impl Dispatcher for Interrupt {}