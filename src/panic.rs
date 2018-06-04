use core::panic::PanicInfo;
use arch::cpu::IrqController;
use arch::interrupt;

#[panic_implementation]
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


#[lang = "eh_personality"] extern fn eh_personality() {}
