use core::fmt;

use x86_64::instructions::port::Port;

bitflags! {
    /// Interrupt enable flags
    struct IntEnFlags: u8 {
        const RECEIVED = 1;
        const SENT = 1 << 1;
        const ERRORED = 1 << 2;
        const STATUS_CHANGE = 1 << 3;
        // 4 to 7 are unused
    }
}

bitflags! {
    /// Line status flags
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        // 1 to 4 unknown
        const OUTPUT_EMPTY = 1 << 5;
        // 6 and 7 unknown
    }
}

pub struct SerialPort {
    data: Port<u8>,
    int_en: Port<u8>,
    fifo_ctrl: Port<u8>,
    line_ctrl: Port<u8>,
    modem_ctrl: Port<u8>,
    line_sts: Port<u8>,
}

impl SerialPort {
    pub const fn new(base: u16) -> SerialPort {
        SerialPort {
            data: Port::new(base),
            int_en: Port::new(base + 1),
            fifo_ctrl: Port::new(base + 2),
            line_ctrl: Port::new(base + 3),
            modem_ctrl: Port::new(base + 4),
            line_sts: Port::new(base + 5),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            self.int_en.write(0x00);
            self.line_ctrl.write(0x80);
            self.data.write(0x03);
            self.int_en.write(0x00);
            self.line_ctrl.write(0x03);
            self.fifo_ctrl.write(0xC7);
            self.modem_ctrl.write(0x0B);
            self.int_en.write(0x01);
        }
    }

    fn line_sts(&self) -> LineStsFlags {
        unsafe {
            LineStsFlags::from_bits_truncate(self.line_sts.read())
        }
    }

    pub fn receive(&mut self) {
        while self.line_sts().contains(LineStsFlags::INPUT_FULL) {
            //
        }
    }

    pub fn send(&mut self, data: u8) {
        unsafe {
            match data {
                8 | 0x7F => {
                    while !self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY) {}
                    self.data.write(8);
                    while !self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY) {}
                    self.data.write(b' ');
                    while !self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY) {}
                    self.data.write(8)
                },
                _ => {
                    while !self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY) {}
                    self.data.write(data);
                }
            }
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
