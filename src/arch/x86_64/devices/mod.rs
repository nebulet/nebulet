
pub mod serial;
pub mod pic;

pub unsafe fn init() {
    pic::init();
}

pub fn init_noncore() {
    // serial::init();
}