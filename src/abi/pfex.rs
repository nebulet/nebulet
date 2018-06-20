use object::{Thread, Event};
use wasm::VmCtx;
use sync::atomic::{Atomic, Ordering};
use nil::Ref;

bitflags! {
    struct PfexFlags: u8 {
        /// Will be set if the pfex is locked.
        const LOCKED = 1 << 0;
        /// Will be set if threads are waiting on this lock.
        const IN_DEMAND = 1 << 1;
    }
}

struct Spinwait {
    counter: u32,
}

impl Spinwait {
    #[inline]
    fn new() -> Spinwait {
        Spinwait {
            counter: 0,
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.counter = 0;
    }

    #[inline]
    fn spin(&mut self) -> bool {
        use core::sync::atomic::spin_loop_hint;
        #[inline]
        fn cpu_relax(iterations: u32) {
            for _ in 0..iterations {
                spin_loop_hint();
            }
        }

        if self.counter >= 20 {
            return false;
        }
        self.counter += 1;
        if self.counter <= 10 {
            cpu_relax(4 << self.counter)
        } else {
            Thread::yield_now();
        }
        true
    }
}

/// This will crash the process when the value_offset doesn't point to committed memory.
/// While somewhat extreme, it is safe.
pub extern fn pfex_acquire(state_offset: u32, vmctx: &VmCtx) {
    let state_ptr: *const Atomic<PfexFlags> = vmctx.fastpath_offset_ptr(state_offset);
    let state = unsafe { &*state_ptr };

    // test for the uncontented case
    if state
        .compare_exchange_weak(PfexFlags::empty(), PfexFlags::LOCKED, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        pfex_slow_lock(&state, state_offset, vmctx);
    }
    // at this point, the pfex will be locked
}

/// This will crash the process when the value_offset doesn't point to committed memory.
/// While somewhat extreme, it is safe.
pub extern fn pfex_release(state_offset: u32, vmctx: &VmCtx) {
    let state_ptr: *const Atomic<PfexFlags> = vmctx.fastpath_offset_ptr(state_offset);
    let state = unsafe { &*state_ptr };

    if state
        .compare_exchange_weak(PfexFlags::LOCKED, PfexFlags::empty(), Ordering::Release, Ordering::Relaxed)
        .is_ok()
    {
        return;
    }
    pfex_slow_unlock(&state, state_offset, vmctx);
    // at this point, the pfex is unlocked
}


fn pfex_slow_lock(state: &Atomic<PfexFlags>, offset: u32, vmctx: &VmCtx) {
    let mut current_state = state.load(Ordering::Relaxed);
    let mut spinwait = Spinwait::new();

    loop {
        // grab the lock if it isn't locked, even if there is a queue on it
        if !current_state.contains(PfexFlags::LOCKED) {
            match state.compare_exchange_weak(
                current_state,
                current_state | PfexFlags::LOCKED,
                Ordering::Acquire,
                Ordering::Relaxed
            ) {
                Ok(_) => return,
                Err(x) => current_state = x,
            }
            continue;
        }

        // If there is no queue, try spinning a few times
        if !current_state.contains(PfexFlags::IN_DEMAND) && spinwait.spin() {
            current_state = state.load(Ordering::Relaxed);
            continue;
        }

        // set the IN_DEMAND flag
        if !current_state.contains(PfexFlags::IN_DEMAND) {
            if let Err(x) = state.compare_exchange_weak(
                current_state,
                current_state | PfexFlags::IN_DEMAND,
                Ordering::Relaxed,
                Ordering::Relaxed
            ) {
                current_state = x;
                continue;
            }
        }

        // block the current thread until woken up.
        {
            let process = &vmctx.data().process;
            
            // if `IN_DEMAND`, then the element
            // in the hashtable already exists
            let event = if current_state.contains(PfexFlags::IN_DEMAND) {
                let pfex_map = process.pfex_map().read();
                pfex_map[&offset].clone()
            } else {
                let mut pfex_map = process.pfex_map().write();
                let new_event = Ref::new(Event::new()).unwrap();
                pfex_map.insert(offset, new_event.clone());
                new_event
            };

            event.wait();

            // loop back and try locking again
            spinwait.reset();
            current_state = state.load(Ordering::Relaxed);
        }
    }    
}

fn pfex_slow_unlock(state: &Atomic<PfexFlags>, offset: u32, vmctx: &VmCtx) {
    if state
        .compare_exchange(PfexFlags::LOCKED, PfexFlags::empty(), Ordering::Release, Ordering::Relaxed)
        .is_ok()
    {
        return;
    }

    {
        let process = &vmctx.data().process;

        let event = {
            let pfex_map = process.pfex_map().read();
            pfex_map[&offset].clone()
        };

        if event.has_queued() {
            state.store(PfexFlags::IN_DEMAND, Ordering::Release);
        } else {
            state.store(PfexFlags::empty(), Ordering::Release);
        }

        event.trigger().unwrap();
    }
}
