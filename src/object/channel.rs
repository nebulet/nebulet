use nil::{Ref, KernelRef};
use nabi::{Result, Error};
use object::Handle;
use alloc::{Vec, VecDeque};
use arch::lock::IrqLock;

pub struct Message {
    data: Vec<u8>,
    handles: Vec<Handle>,
}

impl Message {
    pub fn new(data: &[u8], handles: Vec<Handle>) -> Message {
        Message {
            data: data.to_vec(),
            handles,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn handles(&self) -> &[Handle] {
        &self.handles
    }
}

/// Represents a writable
/// and readable channel
/// for transferring data
/// between processes.
#[derive(KernelRef)]
pub struct Channel {
    msg_buffer: IrqLock<VecDeque<Message>>,
}

impl Channel {
    pub fn new() -> Result<Ref<Self>> {
        Ref::new(Channel {
            msg_buffer: IrqLock::new(VecDeque::new()),
        })
    }

    pub fn write(&self, msg: Message) -> Result<()> {
        self.msg_buffer
            .lock()
            .push_back(msg);

        Ok(())
    }

    pub fn read(&self) -> Result<Message> {
        self.msg_buffer
            .lock()
            .pop_front()
            .ok_or(Error::SHOULD_WAIT)
    }
}
