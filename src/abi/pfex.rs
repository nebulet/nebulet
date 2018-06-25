use object::thread::{Thread, State};
use wasm::VmCtx;
use sync::atomic::{Atomic, Ordering};
use sync::mpsc::IntrusiveMpsc;

/// This will crash the process when the value_offset doesn't point to committed memory.
/// While somewhat extreme, it is safe.
pub extern fn pfex_acquire(lock_offset: u32, vmctx: &VmCtx) {
    let user_data = &vmctx.data().user_data;
    let lock_ptr: *const Atomic<u32> = vmctx.fastpath_offset_ptr(lock_offset);
    let lock = unsafe { &*lock_ptr };

    loop {
        let mut pfex_map = user_data.process.pfex_map().lock();
        let locked = lock.load(Ordering::Relaxed);

        if locked == 0 {
            lock.store(1, Ordering::Release);
            break;
        } else {
            let queue = pfex_map
                .entry(lock_offset)
                .or_insert(IntrusiveMpsc::new());

            let current_thread = Thread::current();

            unsafe { queue.push(current_thread); } // this must be first
            current_thread.set_state(State::Blocked);

            // drop the lock on the pfex_map to avoid deadlocks
            drop(pfex_map);

            Thread::yield_now();
        }
    }
    // at this point, the pfex will be locked
}

/// This will crash the process when the value_offset doesn't point to committed memory.
/// While somewhat extreme, it is safe.
pub extern fn pfex_release(lock_offset: u32, vmctx: &VmCtx) {
    let lock_ptr: *const Atomic<u32> = vmctx.fastpath_offset_ptr(lock_offset);
    let lock = unsafe { &*lock_ptr };

    let user_data = &vmctx.data().user_data;
    let mut pfex_map = user_data.process.pfex_map().lock();
    let locked = lock.load(Ordering::Relaxed);

    if locked != 0 {
        lock.store(0, Ordering::Release);
        if let Some(queue) = pfex_map.remove(&lock_offset) {
            unsafe {
                while let Some(thread) = queue.pop() {
                    (*thread).resume();
                }
            }
        }
    }
    // at this point, the pfex is unlocked
}
