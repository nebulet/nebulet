use alloc::Vec;
use core::mem;
use core::ops::{Index, IndexMut};

use nabi::{Result, Error};

pub type TableIndex = usize;

#[derive(Debug)]
enum Slot<T> {
    Occupied(T),
    /// The index of the next vacant slot
    Vacant(usize),
}

#[derive(Debug)]
pub struct Table<T> {
    slots: Vec<Slot<T>>,
    head: usize,
    len: usize,
}

impl<T> Table<T> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        assert!(capacity > 0, "`capacity` must be greater than 0.");
        Table {
            slots: Vec::with_capacity(capacity),
            head: !0,
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.slots.capacity()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn allocate(&mut self, object: T) -> Result<TableIndex> {
        self.len += 1;

        if self.head != !0 {
            match self.slots[self.head] {
                Slot::Vacant(next) => {
                    self.head = next;
                    self.slots[self.head] = Slot::Occupied(object);
                },
                Slot::Occupied(_) => unreachable!(),
            }
            Ok(self.head)
        } else {
            self.slots.push(Slot::Occupied(object));
            Ok(self.len - 1)
        }
    }

    pub fn free(&mut self, index: TableIndex) -> Result<T> {
        match self.slots.get_mut(index) {
            Some(&mut Slot::Vacant(_)) | None => Err(Error::NOT_FOUND),
            Some(slot @ &mut Slot::Occupied(_)) => {
                if let Slot::Occupied(object) = mem::replace(slot, Slot::Vacant(self.head)) {
                    self.head = index;
                    self.len -= 1;
                    Ok(object)
                } else {
                    unreachable!();
                }
            }
        }
    }

    pub fn get(&self, index: TableIndex) -> Option<&T> {
        match self.slots.get(index) {
            Some(&Slot::Vacant(_)) | None => None,
            Some(&Slot::Occupied(ref obj)) => Some(obj),
        }
    }

    pub fn get_mut(&mut self, index: TableIndex) -> Option<&mut T> {
        match self.slots.get_mut(index) {
            Some(&mut Slot::Vacant(_)) | None => None,
            Some(&mut Slot::Occupied(ref mut obj)) => Some(obj),
        }
    }
}

impl<T> Index<TableIndex> for Table<T> {
    type Output = T;

    fn index(&self, index: TableIndex) -> &T {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<TableIndex> for Table<T> {
    fn index_mut(&mut self, index: TableIndex) -> &mut T { 
        self.get_mut(index).unwrap()
    }
}