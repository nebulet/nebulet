
use sip::abi;
use keyboard::{Keyboard, layouts};

static mut KEYBOARD: Keyboard<layouts::Us104Key> = Keyboard::new();

pub struct KeyboardDriver;

impl KeyboardDriver {
    pub fn new() -> KeyboardDriver {
        unsafe {
            ps2_init();
            keyboard_init();
            abi::set_irq_handler(33, handler);
        }
        KeyboardDriver
    }
}

unsafe extern fn handler() {
    let scancode = abi::read_port_u8(0x60);

    if let Ok(Some(key_event)) = KEYBOARD.add_byte(scancode) {
        if let Some(key) = KEYBOARD.process_keyevent(key_event) {
            println!("{:?}", key);
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
        try_cmd(0xf0)?;
        try_cmd(0x02)
    };

    match set_scanset_2() {
        Err(_) => {
            println!("Unable to set scanset.");
            panic!();
        },
        _ => {},
    }
}
