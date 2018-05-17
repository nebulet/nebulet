pub mod handle;

pub use self::handle::{HandleTable, Handle, HandleRights};
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

static HANDLE_TABLE: Once<RwLock<HandleTable>> = Once::new();

fn handle_table_init() -> RwLock<HandleTable> {
    RwLock::new(HandleTable::new())
}

pub struct GlobalHandleTable;

impl GlobalHandleTable {
    pub fn get() -> RwLockReadGuard<'static, HandleTable> {
        HANDLE_TABLE
            .call_once(handle_table_init)
            .read()
    }

    pub fn get_mut() -> RwLockWriteGuard<'static, HandleTable> {
        HANDLE_TABLE
            .call_once(handle_table_init)
            .write()
    }
}
