
use sip::irq::{create_irq_event, ack_irq};
use sip::thread;
use sip::Mutex;
use sip::abi;
use keyboard::{Keyboard, layouts, DecodedKey};

static EVENT_QUEUE: Mutex<Vec<DecodedKey>> = Mutex::new(Vec::new());

pub struct KeyboardDriver;

impl KeyboardDriver {
    pub fn new() -> KeyboardDriver {
        unsafe {
            ps2_init();
            keyboard_init();
        }

        let mut keyboard: Keyboard<layouts::Us104Key> = Keyboard::new();
        let irq_event = unsafe { create_irq_event(33).unwrap() };

        thread::spawn(move || {
            loop {
                irq_event.wait().unwrap();
                irq_event.rearm().unwrap();
                let scancode = unsafe { abi::read_port_u8(0x60) };

                if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                    if let Some(key) = keyboard.process_keyevent(key_event) {
                        EVENT_QUEUE
                            .lock()
                            .push(key);
                    }
                }
                unsafe { ack_irq(33).unwrap(); }
            }
        }).unwrap();

        KeyboardDriver
    }

    pub fn get_key(&self) -> Option<DecodedKey> {
        let mut queue = EVENT_QUEUE.lock();

        if queue.len() > 0 {
            Some(queue.remove(0))
        } else {
            None
        }
    }
}

unsafe fn ps2_init() {
    let wait_read = || while abi::read_port_u8(0x64) & 1 == 0 {};
    let wait_write = || while abi::read_port_u8(0x64) & 1 << 1 != 0 {};

    // turn translation off
    abi::write_port_u8(0x64, 0x20);
    wait_read();
    let mut config = abi::read_port_u8(0x60);
    config &= !(1 << 6);
    abi::write_port_u8(0x64, 0x60);
    wait_write();
    abi::write_port_u8(0x60, config);
}

unsafe fn keyboard_init() {
    let wait_read = || while abi::read_port_u8(0x64) & 1 == 0 {};
    let wait_write = || while abi::read_port_u8(0x64) & 1 << 1 != 0 {};

    let try_cmd = |cmd| {
        loop {
            wait_write();
            abi::write_port_u8(0x60, cmd);
            wait_read();
            match abi::read_port_u8(0x60) {
                0xfa => break,
                0xfe => {},
                _ => return Err(()),
            }
        }
        Ok(())
    };

    let set_scanset_2 = || {
        try_cmd(0xf5)?;
        try_cmd(0xf0)?;
        try_cmd(0x02)?;
        try_cmd(0xf4)
    };

    match set_scanset_2() {
        Err(_) => {
            println!("Unable to set scanset.");
            panic!();
        },
        _ => {},
    }
}
