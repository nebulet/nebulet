use common::devices::uart_16550::SerialPort;
use arch::lock::PreemptLock;

pub static COM1: PreemptLock<SerialPort> = PreemptLock::new(SerialPort::new(0x3F8));
pub static COM2: PreemptLock<SerialPort> = PreemptLock::new(SerialPort::new(0x2F8));

pub unsafe fn init() {
    COM1.lock().init();
    COM2.lock().init();
}

pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    let _ = COM1.lock().write_fmt(args);
}