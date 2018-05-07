//! The interface between running processes and the kernel
//!

use arch::lock::IrqLock;

static LOCK: IrqLock<()> = IrqLock::new(());

pub extern fn output_test(arg: usize) {
    let guard = LOCK.lock();
    println!("wasm supplied arg = {}", arg);
    guard.release();
}
