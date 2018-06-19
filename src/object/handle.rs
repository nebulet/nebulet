use nabi::{Result, Error, HandleRights};
use nil::{Ref, HandleRef};
use core::marker::PhantomData;
use core::ops::Deref;

/// A Handle represents an atomically reference-counted object with specfic rights.
/// Handles can be duplicated if they have the `HandleRights::DUPLICATE` right.
pub struct Handle<T: HandleRef + ?Sized> {
    /// Reference-counted ptr to the stored object.
    refptr: Ref<T>,
    /// This handle's access rights to the `Ref<T>`.
    rights: HandleRights,
}

impl<T: HandleRef + ?Sized> Handle<T>
{
    pub fn new(refptr: Ref<T>, rights: HandleRights) -> Handle<T> {
        Handle {
            refptr,
            rights,
        }
    }
    
    pub fn duplicate(&self, new_rights: HandleRights) -> Option<Self> {
        if self.rights.contains(new_rights | HandleRights::DUPLICATE) {
            Some(Handle {
                refptr: self.refptr.clone(),
                rights: new_rights,
            })
        } else {
            None
        }
    }

    pub fn check_rights(&self, rights: HandleRights) -> Result<&Self> {
        if self.rights.contains(rights) {
            Ok(self)
        } else {
            Err(Error::ACCESS_DENIED)
        }
    }

    pub fn rights(&self) -> HandleRights {
        self.rights
    }

    pub fn refptr(self) -> Ref<T> {
        self.refptr
    }
}

impl<T> Handle<T>
where
    T: HandleRef + Sized
{
    pub fn upcast(self) -> Handle<HandleRef> {
        Handle {
            refptr: self.refptr,
            rights: self.rights,
        }
    }
}

impl Handle<HandleRef> {
    pub fn cast<T: HandleRef>(&self) -> Result<Handle<T>> {
        let refptr = self.refptr.cast()
            .ok_or(Error::WRONG_TYPE)?;

        Ok(Handle {
            refptr,
            rights: self.rights,
        })
    }
}

impl<T> Deref for Handle<T>
where
    T: HandleRef + ?Sized
{
    type Target = Ref<T>;
    fn deref(&self) -> &Ref<T> {
        &self.refptr
    }
}

#[repr(transparent)]
pub struct UserHandle<T: HandleRef + ?Sized>(u32, PhantomData<T>);

impl<T> UserHandle<T>
where
    T: HandleRef + ?Sized
{
    #[inline]
    pub fn new(index: u32) -> UserHandle<T> {
        UserHandle(index, PhantomData)
    }

    #[inline]
    pub fn inner(&self) -> u32 {
        self.0
    }
}
