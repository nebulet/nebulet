use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use devices::pic;
use time;
use context;
use macros::{interrupt, println};
use x86_64::instructions::port::Port;

pub static PIT_TICKS: AtomicUsize = ATOMIC_USIZE_INIT;
static CONTEXT_SWITCH_TICKS: usize = 10;

#[inline]
unsafe fn trigger(irq: u8) {
    if irq < 16 {
        println!("acknowledged");
        if irq >= 8 {
            // pic::SLAVE.mask_set(irq - 8);
            pic::MASTER.ack();
            pic::SLAVE.ack();
        } else {
            // pic::MASTER.mask_set(irq);
            pic::MASTER.ack();
        }
    }

    // TODO: Actually do something
}

interrupt!(pit, {
    const PIT_RATE: u64 = 2250286;

    {
        let mut offset = time::OFFSET.lock();
        let sum = offset.1 + PIT_RATE;
        offset.1 = sum % 1_000_000_000;
        offset.0 += sum / 1_000_000_000;
    }

    // Saves CPU time by shortcutting
    pic::MASTER.ack();

    // switch context
    if PIT_TICKS.fetch_add(1, Ordering::SeqCst) >= CONTEXT_SWITCH_TICKS {
        context::SCHEDULER.switch();
        PIT_TICKS.store(0, Ordering::SeqCst);
    }
});

interrupt!(keyboard, {
    let scancode = unsafe { Port::<u8>::new(0x60).read() };
    println!("keyboard interrupt: {}", scancode);

    trigger(1);
});

interrupt!(rtc, {
    println!("RTC interrupt");
    
    trigger(8);
});