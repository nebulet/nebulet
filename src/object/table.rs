use super::{Handle, HandleRights};
use nil::{Ref, KernelRef};
use nil::mem::Array;

use nabi::{Result, Error};

pub struct HandleTable {
    /// Raw array of handles,
    array: Array<Option<Handle>>,
    /// Stack/queue of free indices.
    free_indices: Array<usize>,
}

impl HandleTable {
    pub fn new() -> HandleTable {
        HandleTable {
            array: Array::new(),
            free_indices: Array::new(),
        }
    }

    pub fn get(&self, index: usize) -> Result<&Handle> {
        self.array.get(index)
            .and_then(|opt| opt.as_ref())
            .ok_or(Error::NOT_FOUND)
    }

    /// This makes a copy of the supplied handle
    /// and inserts it into `self`.
    pub fn transfer_handle(&mut self, handle: Handle) -> Result<usize> {
        if handle.rights().contains(HandleRights::TRANSFER) {
            self.allocate_handle(handle)
        } else {
            Err(Error::ACCESS_DENIED)
        }
    }

    fn allocate_handle(&mut self, handle: Handle) -> Result<usize> {
        if let Some(index) = self.free_indices.pop() {
            debug_assert!(self.array[index].is_none());
            self.array[index] = Some(handle);
            Ok(index)
        } else {
            self.array.push(Some(handle))?;
            Ok(self.array.len() - 1)
        }
    }

    pub fn allocate<T: KernelRef>(&mut self, refptr: Ref<T>, rights: HandleRights) -> Result<usize> {
        let handle = Handle::new(refptr, rights);
        self.allocate_handle(handle)
    }

    pub fn free(&mut self, index: usize) -> Result<Handle> {
        let handle = self.array.replace_at(index, None)
            .and_then(|opt| opt)
            .and_then(|handle| Some(handle))
            .ok_or(Error::NOT_FOUND)?;
        
        self.free_indices.push(index)?;
        Ok(handle)
    }

    pub fn duplicate(&mut self, index: usize, new_rights: HandleRights) -> Result<usize> {
        let dup = {
            let handle = self.get(index)?;
            handle.duplicate(new_rights)
                    .ok_or(Error::ACCESS_DENIED)
        }?;

        self.allocate_handle(dup)
    }
}
