use arch;
use task;
use interrupt;
use time::{Instant, Duration, INSTANT_INIT};

use alloc::boxed::Box;
use alloc::arc::Arc;
use core::cmp::{max, min};
use core::ops::{Deref, DerefMut};

use nabi::Result;

use macros::println;

use spin::RwLock;

bitflags! {
    pub struct ThreadFlags: u32 {
        const DETACHED  = 1 << 0;
        const REAL_TIME = 1 << 3;
        const IDLE      = 1 << 4;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ThreadPriority {
    pub base: u8,
    pub boost: u8,
    pub effective: u8,
    pub inherited: u8,
}

impl ThreadPriority {
    pub const NUM: usize    = 32;
    pub const LOWEST: u8    = 0;
    pub const HIGHEST: u8   = ThreadPriority::NUM as u8 - 1;
    pub const IDLE: u8      = ThreadPriority::LOWEST;
    pub const LOW: u8       = ThreadPriority::NUM as u8 / 4;
    pub const DEFAULT: u8   = ThreadPriority::NUM as u8 / 2;
    pub const HIGH: u8      = (ThreadPriority::NUM as u8 / 4) * 3;

    pub const BOOST_MAX_DIFF: u8 = 4;

    pub fn new(base: u8) -> ThreadPriority {
        ThreadPriority {
            base: base,
            boost: 0,
            effective: base,
            inherited: 0,
        }
    }

    pub fn boost(&mut self) {
        if self.boost < Self::BOOST_MAX_DIFF
        && likely!(self.base + self.boost < Self::HIGHEST) {
            self.boost += 1;
        }
        
        self.compute_effective();
    }

    pub fn deboost(&mut self, quantum_expired: bool) {
        let boost_floor: i16 = if quantum_expired {
            if self.base - Self::BOOST_MAX_DIFF < Self::LOWEST {
                self.base as i16 - Self::LOWEST as i16
            } else {
                -1
            }
        } else {
            0
        };

        if self.boost as i16 <= boost_floor as i16 {
            return;
        }
        self.boost -= 1;

        self.compute_effective();
    }

    pub fn compute_effective(&mut self) {
        self.effective = max(self.inherited, self.base + self.boost);

        debug_assert!(self.effective >= Self::LOWEST && self.effective <= Self::HIGHEST);
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ThreadState {
    Suspended,
    Ready,
    Running,
    Blocked,
    Sleeping,
    Dead,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Thread {
    /// Stack stuff
    pub stack: Box<[u8]>,

    pub arch: arch::Context,

    pub priority: ThreadPriority,

    pub state: ThreadState,

    /// Entry point
    pub entry: extern fn(usize) -> i32,
    pub arg: usize,

    /// Return code
    pub retcode: i32,

    /// Maximum thread name length is 32 bytes
    pub name: [u8; 32],

    pub flags: ThreadFlags,

    /// The remaining time slice
    pub remaining_time_slice: Duration,

    /// The instant that the thread last started running
    pub last_started_running: Instant,

    /// The total thread runtime
    pub runtime: Duration,
}

impl Thread {
    /// Threads get 10 millis to run before they use up their time slice and get preempted
    pub const INTIAL_TIME_SLICE: Duration = Duration::from_millis(10);
}

#[derive(Debug, Clone)]
pub struct LockedThread(Arc<RwLock<Thread>>);

impl Deref for LockedThread {
    type Target = Arc<RwLock<Thread>>;

    fn deref(&self) -> &Arc<RwLock<Thread>> {
        &self.0
    }
}

impl DerefMut for LockedThread {
    fn deref_mut(&mut self) -> &mut Arc<RwLock<Thread>> {
        &mut self.0
    }
}

impl LockedThread {
    /// Wrap a thread in a LockedThread
    pub fn capture(thread: Thread) -> LockedThread {
        LockedThread(Arc::new(RwLock::new(thread)))
    }

    pub fn create(name: &str, entry: extern fn(usize) -> i32, arg: usize, stack_size: usize) -> Result<LockedThread> {
        let stack = vec![0; stack_size].into_boxed_slice();

        let mut name_buf = [0; 32];

        // copy name into name
        let len = min(name.len(), name_buf.len());
        name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);

        let mut thread = Thread {
            stack: stack,
            arch: arch::Context::new(),
            priority: ThreadPriority::new(ThreadPriority::DEFAULT),
            state: ThreadState::Suspended,
            entry: entry,
            arg: arg,
            retcode: 0,
            name: name_buf,
            flags: ThreadFlags::empty(),
            remaining_time_slice: Duration::from_secs(0),
            last_started_running: INSTANT_INIT,
            runtime: Duration::from_secs(0),
        };

        // initialize thread
        unsafe {
            arch::thread_initialize(&mut thread);
        }

        Ok(LockedThread::capture(thread))
    }

    pub fn real_time(&self) -> bool {
        self.read()
            .flags.contains(ThreadFlags::REAL_TIME)
    }

    pub fn idle(&self) -> bool {
        self.read()
            .flags.contains(ThreadFlags::IDLE)
    }

    pub fn detached(&self) -> bool {
        self.read()
            .flags.contains(ThreadFlags::DETACHED)
    }

    pub fn resume(&self) -> Result<usize> {
        {
            let mut t = self.write();

            if t.state == ThreadState::Suspended {
                t.state = ThreadState::Ready;
            } else if t.state == ThreadState::Dead {
                // resuming a "dead" thread does nothing
                return Ok(0)
            }
        }

        {
            let t = self.read();

            if t.remaining_time_slice > Duration::from_secs(0) {
                task::scheduler().insert_thread_front(&self);
            } else {
                task::scheduler().insert_thread_back(&self);
            }
        }

        Ok(0)
    }
}

pub extern fn idle_thread_entry(_:usize) -> i32 {
    println!("in idle thread");
    loop {
        unsafe { interrupt::halt(); }
    }
}