pub mod serial;
pub mod pic;
pub mod rtc;
pub mod pit;
pub mod high_precision_timer;
pub mod rand;

pub unsafe fn init() {
    pic::init();
}

pub unsafe fn init_noncore() {
    pit::init();
    rtc::init();
    #[cfg(feature = "serial")]
    serial::init();
    high_precision_timer::init();
}
