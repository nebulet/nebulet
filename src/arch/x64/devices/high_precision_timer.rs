use arch::x64::interrupt;
use arch::x64::idt;
use arch::macros::interrupt;
use arch::devices::{pic, pit};
use sync::atomic::{Atomic, Ordering};
use x86_64::structures::idt::InterruptDescriptorTable;
use core::mem::transmute;

/// Cycles per second according to rdtsc (which is generally a maximum
/// cpu frequency).
static mut TSC_RATE: u64 = 0;

static PIT_TICKS: Atomic<u8> = Atomic::new(0);

fn custom_idt() -> InterruptDescriptorTable {
    interrupt!(pit, {
        PIT_TICKS.fetch_add(1, Ordering::SeqCst);
        // Saves CPU time by shortcutting
        pic::MASTER.ack();
    });

    let mut idt = idt::default_idt();
    idt[32].set_handler_fn(pit);
    idt
}

pub fn rdtsc() -> u64 {
    unsafe {
        let lower: u64;
        let higher: u64;
        asm!("
            lfence
            rdtsc"
            : "={edx}"(higher), "={eax}"(lower)
        );

        higher << 32 | lower
    }
}

pub unsafe fn init() {
    let idt = custom_idt();
    let idt_ref = transmute::<&InterruptDescriptorTable, &'static InterruptDescriptorTable>(&idt);
    idt_ref.load();
    interrupt::enable();

    let initial_tick = PIT_TICKS.load(Ordering::SeqCst);

    // Wait until the start of a cycle
    let mut start_tick;
    let mut start_cycle;
    loop {
        start_tick = PIT_TICKS.load(Ordering::SeqCst);
        start_cycle = rdtsc();
        if initial_tick != start_tick {
            break;
        }
    }

    // Wait one tick cycle;
    let mut end_tick;
    let mut end_cycle;
    loop {
        end_tick = PIT_TICKS.load(Ordering::SeqCst);
        end_cycle = rdtsc();
        if end_tick != start_tick {
            break;
        }
    }

    // Restore interrupts
    interrupt::disable();
    idt::IDT.load();

    // Calculate and store frequency
    assert_eq!(start_tick + 1, end_tick);

    let cycles_per_billion_pits: u64 = 1_000_000_000 * (end_cycle - start_cycle);
    let cycles_per_second = cycles_per_billion_pits / pit::RATE as u64;
    TSC_RATE = cycles_per_second;
}

/// Time from arbitrary epoch in nano seconds
pub fn now() -> u64 {
    let cycle = rdtsc();
    let rate = unsafe{ TSC_RATE };

    cycle.wrapping_mul(1_000_000_000) / rate
}
