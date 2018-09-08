use arch::cpu::IrqController;
use arch::interrupt;
use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    unsafe {
        IrqController::disable();
        loop {
            interrupt::halt();
        }
    }
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
