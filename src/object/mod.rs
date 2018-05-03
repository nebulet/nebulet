pub mod handle;

pub use self::handle::{HandleTable, Handle};
use spin::Once;

use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub static HANDLE_TABLE: Once<RwLock<HandleTable>> = Once::new();

fn handle_table_init() -> RwLock<HandleTable> {
    RwLock::new(HandleTable::new())
}

pub fn handle_table() -> RwLockReadGuard<'static, HandleTable> {
    HANDLE_TABLE
        .call_once(handle_table_init)
        .read()
}

pub fn handle_table_mut() -> RwLockWriteGuard<'static, HandleTable> {
    HANDLE_TABLE
        .call_once(handle_table_init)
        .write()
}