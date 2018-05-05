use common::table::TableIndex;
use super::GlobalScheduler;
use nabi::Result;

/// `ThreadEntry` is an opaque wrapper around
/// an index into the global thread table.
/// 
/// Using indexes into a table instead of 
/// atomic references is an attempt at
/// reducing the overhead of thread
/// handling, in terms of both raw performance
/// and programmer time.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ThreadEntry(pub(super) TableIndex);

impl ThreadEntry {
    #[inline]
    pub(super) fn id(&self) -> TableIndex {
        self.0
    }

    pub fn resume(&self) -> Result<()> {
        GlobalScheduler::push(*self);

        Ok(())
    }
}