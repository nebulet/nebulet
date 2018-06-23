use alloc::Vec;
use core::mem;
use core::ops::{Index, IndexMut};

pub type TableIndex = usize;

#[derive(Debug)]
pub struct Table<T> {
    objects: Vec<T>,
    free_list: Vec<usize>,
}

impl<T> Table<T> {
    pub fn new() -> Self {
        Table {
            objects: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        assert!(capacity > 0, "`capacity` must be greater than 0.");
        Table {
            objects: Vec::with_capacity(capacity),
            free_list: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn allocate(&mut self, object: T) -> Option<TableIndex> {
        if let Some(index) = self.free_list.pop() {
            unsafe {
                (&mut self.objects[index] as *mut T).write(object);
            }
            Some(index)
        } else {
            self.objects.push(object);
            Some(self.objects.len() - 1)
        }
    }

    pub fn free(&mut self, index: TableIndex) -> Option<T> {
        if self.objects.len() > index {
            Some(unsafe {
                (&mut self.objects[index] as *mut T).replace(mem::uninitialized())
            })
        } else {
            None
        }
    }

    pub fn get(&self, index: TableIndex) -> Option<&T> {
        self.objects.get(index)
    }

    pub fn get_mut(&mut self, index: TableIndex) -> Option<&mut T> {
        self.objects.get_mut(index)
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
