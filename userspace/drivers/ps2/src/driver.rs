
use sip::interrupt::Interrupt;
use sip::{Channel, ReadChannel};
use sip::thread;
use sip::abi;
use super::keyboard::{Keyboard, layouts, DecodedKey};
use std::{slice, mem};

pub struct KeyboardDriver {
    key_rx: ReadChannel,
}

impl KeyboardDriver {
    pub fn open() -> KeyboardDriver {
        unsafe {
            ps2_init();
            keyboard_init();
        }

        let mut keyboard: Keyboard<layouts::Us104Key> = Keyboard::new();
        
        let (packet_tx, packet_rx) = Channel::create().unwrap();
        let interrupt = Interrupt::create(packet_tx, 33).unwrap();
        let (key_tx, key_rx) = Channel::create().unwrap();

        thread::spawn(move || {
            let mut packet_buffer = [0u8; 16];

            loop {
                let (length, res) = packet_rx.recv_raw(&mut packet_buffer);
                res.unwrap();
                assert!(length == 16);

                let scancode = unsafe { abi::read_port_u8(0x60) };

                if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                    if let Some(key) = keyboard.process_keyevent(key_event) {
                        let ptr = &key as *const DecodedKey as *const u8;
                        let data = unsafe { slice::from_raw_parts(ptr, mem::size_of::<DecodedKey>()) };
                        key_tx.send(data).unwrap();
                    }
                }
                interrupt.ack().unwrap();
            }
        }).unwrap();

        KeyboardDriver {
            key_rx,
        }
    }

    pub fn keys(self) -> Iter {
        Iter {
            key_rx: self.key_rx,
        }
    }
}

pub struct Iter {
    key_rx: ReadChannel,
}

impl Iterator for Iter {
    type Item = DecodedKey;
    fn next(&mut self) -> Option<DecodedKey> {
        let mut buffer = [0u8; mem::size_of::<DecodedKey>()];

        let (_, res) = self.key_rx.recv_raw(&mut buffer);

        match res {
            Ok(_) => {
                let ptr = &buffer as *const [u8] as *const DecodedKey;
                let key = unsafe { *ptr };
                Some(key)
            },
            Err(_) => None,
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
