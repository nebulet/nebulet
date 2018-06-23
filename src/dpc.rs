use object::Thread;
use sync::mpsc::Mpsc;

static DPC_QUEUE: Mpsc<(usize, fn(usize))> = Mpsc::new();

pub fn init() {
    let dpc_thread = Thread::new(4096 * 4, || {
        loop {
            while let Some((arg, f)) = unsafe { DPC_QUEUE.pop() } {
                f(arg);
            }
            Thread::yield_now();
        }
    }).expect("unable to initialize dpc functionality");

    dpc_thread.resume();
}

/// Queue a function to be called at some point.
/// The function is run once.
pub fn queue(arg: usize, f: fn(usize))
{
    DPC_QUEUE.push((arg, f));
}
