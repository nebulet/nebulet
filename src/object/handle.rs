use nabi::{Result, Error};
use nil::{Ref, KernelRef};

bitflags! {
    pub struct HandleRights: u32 {
        /// The right to duplicate a handle;
        const DUPLICATE = 1 << 0;
        /// The right to transfer a handle
        /// to another process.
        const TRANSFER  = 1 << 1;
        /// The right to read the object
        /// refered to by a handle.
        const READ      = 1 << 2;
        /// The right to read the object
        /// refered to by a handle.
        const WRITE     = 1 << 3;
    }
}

/// A Handle represents an atomically reference-counted object with specfic rights.
/// Handles can be duplicated if they have the `HandleRights::DUPLICATE` right.
pub struct Handle {
    /// Reference-counted ptr to the stored kernel object.
    refptr: Ref<KernelRef>,
    /// This handle's access rights to `Ref`.
    rights: HandleRights,
}

impl Handle {
    pub fn new<T: KernelRef>(refptr: Ref<T>, rights: HandleRights) -> Handle {
        Handle {
            refptr,
            rights,
        }
    }

    pub fn cast<T: KernelRef>(&self) -> Result<&T> {
        self.refptr.cast()
            .ok_or(Error::WRONG_TYPE)
    }

    /// Duplicate the handle if it has the `DUPLICATE` right.
    pub fn duplicate(&self, new_rights: HandleRights) -> Option<Handle> {
        if self.rights.contains(new_rights | HandleRights::DUPLICATE) {
            // `new_rights` contains the same or fewer rights and `HandleRights::DUPLICATE`
            // so it's okay to duplicate it.
            Some(Handle {
                refptr: self.refptr.clone(),
                rights: new_rights,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn rights(&self) -> HandleRights {
        self.rights
    }
}
