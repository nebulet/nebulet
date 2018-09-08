//! Nebulet ABI

/// ABIs for drivers
pub mod driver;
/// ABIs for events
pub mod event;
/// ABIs for manipulating handles
pub mod handle;
/// ABIs for irqs
pub mod interrupt;
/// Intrinsics
pub mod intrinsics;
/// ABIs for I/O
pub mod io;
/// ABIs for IPC
pub mod ipc;
/// ABIs for interfacing with generic objects
pub mod object;
/// ABIs for pretty fast exclusion
pub mod pfex;
/// ABIs for working with processes.
pub mod process;
/// ABIs for random numbers
pub mod rand;
/// Various test ABIs.
pub mod test;
/// ABIs for threads
pub mod thread;
// /// ABIs for services
// pub mod service;
