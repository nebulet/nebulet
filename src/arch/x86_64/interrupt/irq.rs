use devices::pic;
use macros::{interrupt, println};

unsafe fn trigger(irq: u8) {
    if irq < 16 {
        if irq >= 8 {
            pic::SLAVE.mask_set(irq - 8);
            pic::MASTER.ack();
            pic::SLAVE.ack();
        } else {
            pic::MASTER.mask_set(irq);
            pic::MASTER.ack();
        }
    }

    // Actually do something
}

interrupt!(keyboard, {
    println!("keyboard interrupt");
    trigger(1);
});