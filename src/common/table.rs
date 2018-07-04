use alloc::vec::{Vec, Drain};
use core::iter::FilterMap;
use core::ops::{Index, IndexMut, RangeBounds};
use core::marker::PhantomData;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct TableSlot(usize);

impl TableSlot {
    pub fn inner(&self) -> usize {
        self.0
    }

    pub fn invalid() -> TableSlot {
        TableSlot(!0)
    }

    pub fn from_usize(index: usize) -> TableSlot {
        TableSlot(index)
    }
}

pub struct Entry<'table, T: 'table> {
    table: *mut Table<T>,
    slot: TableSlot,
    _phantom: PhantomData<&'table ()>,
}

impl<'table, T: 'table> Entry<'table, T> {
    pub fn remove(self) -> T {
        unsafe {
            (*self.table).free(self.slot).unwrap()
        }
    }

    pub fn get(&self) -> &T {
        unsafe {
            (*self.table).get(self.slot).unwrap()
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe {
            (*self.table).get_mut(self.slot).unwrap()
        }
    }
}

#[derive(Debug)]
pub struct Table<T> {
    objects: Vec<Option<T>>,
    free_list: Vec<usize>,
    len: usize,
}

impl<T> Table<T> {
    pub fn new() -> Self {
        Table {
            objects: Vec::new(),
            free_list: Vec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Table {
            objects: Vec::with_capacity(capacity),
            free_list: Vec::new(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn next_slot(&self) -> TableSlot {
        if let Some(index) = self.free_list.last() {
            TableSlot(*index)
        } else {
            TableSlot(self.objects.len())
        }
    }

    pub fn allocate(&mut self, object: T) -> TableSlot {
        self.len += 1;
        if let Some(index) = self.free_list.pop() {
            self.objects[index] = Some(object);
            TableSlot(index)
        } else {
            self.objects.push(Some(object));
            TableSlot(self.objects.len() - 1)
        }
    }

    pub fn free(&mut self, slot: TableSlot) -> Option<T> {
        if let Some(opt) = self.objects.get_mut(slot.0) {
            opt.take()
        } else {
            None
        }
    }

    pub fn get(&self, slot: TableSlot) -> Option<&T> {
        self.objects.get(slot.0).and_then(|item| item.as_ref())
    }

    pub fn get_mut(&mut self, slot: TableSlot) -> Option<&mut T> {
        self.objects.get_mut(slot.0).and_then(|item| item.as_mut())
    }

    pub fn drain<R>(&mut self, range: R) -> FilterMap<Drain<Option<T>>, impl FnMut(Option<T>) -> Option<T>>
        where R: RangeBounds<usize>
    {
        self.objects.drain(range).filter_map(|item| item)
    }

    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.objects.iter().filter_map(|item| item.as_ref())
    }

    pub fn slot_iter(&self) -> impl Iterator<Item=TableSlot> + '_ {
        self.objects.iter().enumerate().filter_map(|(index, item)| {
            if item.is_some() {
                Some(TableSlot(index))
            } else {
                None
            }
        })
    }

    pub fn entries<'a>(&'a mut self) -> impl Iterator<Item=Entry<T>> + 'a {
        let table = self as *mut _;
        self.objects.iter().enumerate().filter_map(move |(index, item)| {
            if item.is_some() {
                Some(Entry {
                    table,
                    slot: TableSlot(index),
                    _phantom: PhantomData,
                })
            } else {
                None
            }
        })
    }
}

impl<T> Index<TableSlot> for Table<T> {
    type Output = T;

    fn index(&self, slot: TableSlot) -> &T {
        self.get(slot).unwrap()
    }
}

impl<T> IndexMut<TableSlot> for Table<T> {
    fn index_mut(&mut self, slot: TableSlot) -> &mut T { 
        self.get_mut(slot).unwrap()
    }
}
