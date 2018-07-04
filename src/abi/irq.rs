use x86_64::structures::idt::ExceptionStackFrame;
use arch::idt;
use arch::devices::pic;
use nabi::{Result, Error};

use object::{Dispatch, EventDispatcher, HandleRights};
use wasm::UserData;
use signals::Signal;
use nebulet_derive::nebulet_abi;

/// This returns an event that will get unblock when the specified
/// irq fires. When thread priorities are implemented, the priority
/// of the unblocked thread will be set to the highest priority.
#[nebulet_abi]
pub fn create_irq_event(irq: u32, user_data: &UserData) -> Result<u32> {
    if irq >= 256 {
        return Err(Error::INVALID_ARG);
    }

    let event = EventDispatcher::new();

    let handle = {
        let mut handle_table = user_data.process.handle_table().write();
        let flags = HandleRights::WRITE | HandleRights::READ | HandleRights::TRANSFER;
        handle_table.allocate(event.copy_ref(), flags)?
    };

    let index = irq as usize - pic::MASTER_OFFSET as usize;

    unsafe {
        EVENT_TABLE[index] = Some(event);
        let handler = IDT_HANDLER[index];
        let mut guard = idt::IdtGuard::new();
        guard[index].set_handler_fn(handler);
        
        let irq = index as u8;
        if irq < 8 {
            pic::MASTER.mask_clear(irq);
        } else {
            pic::SLAVE.mask_clear(irq);
        }
    }

    Ok(handle.inner())
}

#[nebulet_abi]
pub fn ack_irq(irq: u32, _: &UserData) -> Result<u32> {
    if irq >= 256 {
        return Err(Error::INVALID_ARG);
    }

    let irq = irq as u8 - pic::MASTER_OFFSET;

    unsafe {
        if irq < 16 {
            if irq >= 8 {
                pic::MASTER.ack();
                pic::SLAVE.ack();
            } else {
                pic::MASTER.ack();
            }
        }

        if irq < 8 {
            pic::MASTER.mask_clear(irq);
        } else {
            pic::SLAVE.mask_clear(irq);
        }
    }

    Ok(0)
}

// #[nebulet_abi]
// pub fn remove_irq_event(event: UserHandle<Event>, user_data: &UserData) -> Result<u32> {

// }

macro_rules! double {
    ([] $out:tt) => {
        $out
    };
    
    ([$tok:tt $($toks:tt)*] [$($e:expr),*]) => {
        double!([$($toks)*] [$($e,)* $($e),*])
    };

    ($e:expr, $($toks:tt)*) => {
        double!([$($toks)*] [$e])
    }
}

static mut EVENT_TABLE: [Option<Dispatch<EventDispatcher>>; 16] = double!(None, !!!!);

macro_rules! idt_handlers {
    ($($name:ident ( $value:expr) ),*) => {
        [$( {
                extern "x86-interrupt" fn $name(_: &mut ExceptionStackFrame) {
                    unsafe {
                        let irq = $value - pic::MASTER_OFFSET;
                        if irq < 8 {
                            pic::MASTER.mask_set(irq);
                        } else {
                            pic::SLAVE.mask_set(irq);
                        }

                        if let Some(ref event) = EVENT_TABLE[irq as usize] {
                            let _ = event.signal(Signal::EVENT_SIGNALED, Signal::empty());
                        }
                    }
                }
                $name
            }
         ),*
        ]
    }
}

static IDT_HANDLER: [extern "x86-interrupt" fn(&mut ExceptionStackFrame); 16] = idt_handlers! {
    idt_handler_0x20 ( 0x20 ),
    idt_handler_0x21 ( 0x21 ),
    idt_handler_0x22 ( 0x22 ),
    idt_handler_0x23 ( 0x23 ),
    idt_handler_0x24 ( 0x24 ),
    idt_handler_0x25 ( 0x25 ),
    idt_handler_0x26 ( 0x26 ),
    idt_handler_0x27 ( 0x27 ),
    idt_handler_0x28 ( 0x28 ),
    idt_handler_0x29 ( 0x29 ),
    idt_handler_0x2a ( 0x2a ),
    idt_handler_0x2b ( 0x2b ),
    idt_handler_0x2c ( 0x2c ),
    idt_handler_0x2d ( 0x2d ),
    idt_handler_0x2e ( 0x2e ),
    idt_handler_0x2f ( 0x2f )
};
