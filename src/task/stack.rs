use memory::{Region, sip};

use nabi::{Result, Error};

#[derive(Debug)]
pub struct Stack {
    region: Region,
}

impl Stack {
    pub fn with_size(size: usize) -> Result<Stack> {
        let region = sip::allocate_region(size)
            .ok_or(Error::NO_MEMORY)?;
        
        Ok(Stack {
            region,
        })
    }

    unsafe fn as_mut_ptr(&self) -> *mut u8 {
        self.region.start().as_u64() as *mut _
    }

    pub fn top(&self) -> *mut u8 {
        unsafe { self.as_mut_ptr().add(self.region.size()) }
    }
}