use alloc::vec::{Vec, Drain};
use core::iter::FilterMap;
use core::ops::{Index, IndexMut, RangeBounds};

pub type TableIndex = usize;

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

    pub fn next_index(&self) -> TableIndex {
        if let Some(index) = self.free_list.last() {
            *index
        } else {
            self.objects.len()
        }
    }

    pub fn allocate(&mut self, object: T) -> TableIndex {
        self.len += 1;
        if let Some(index) = self.free_list.pop() {
            self.objects[index] = Some(object);
            index
        } else {
            self.objects.push(Some(object));
            self.objects.len() - 1
        }
    }

    pub fn free(&mut self, index: TableIndex) -> Option<T> {
        self.len -= 1;
        if self.objects.len() > index {
            self.objects[index].take()
        } else {
            None
        }
    }

    pub fn get(&self, index: TableIndex) -> Option<&T> {
        self.objects.get(index).and_then(|item| item.as_ref())
    }

    pub fn get_mut(&mut self, index: TableIndex) -> Option<&mut T> {
        self.objects.get_mut(index).and_then(|item| item.as_mut())
    }

    pub fn drain<R>(&mut self, range: R) -> FilterMap<Drain<Option<T>>, impl FnMut(Option<T>) -> Option<T>>
        where R: RangeBounds<usize>
    {
        self.objects.drain(range).filter_map(|item| item)
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
