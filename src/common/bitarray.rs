
use core::mem;
use alloc::vec::Vec;

use core::ops::Range;

/// A constant-size dense array of bits
#[derive(Debug, PartialEq, Eq)]
pub struct BitArray {
    storage: Vec<u64>,
    nbits: usize,
}

impl BitArray {
    pub fn new(size: usize) -> BitArray {
        let bits = mem::size_of::<u64>() * 8;
        let mut storage = Vec::new();
        storage.resize((size / bits) + 1, 0);
        BitArray {
            storage: storage,
            nbits: size,
        }
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        if index >= self.nbits {
            None
        } else {
            let bits = mem::size_of::<u64>() * 8;
            let w = index / bits;
            let b = index % bits;
            self.storage.get(w).map(|&block|
                (block & (1 << b)) != 0
            )
        }
    }

    pub fn set(&mut self, index: usize, v: bool) {
        assert!(index < self.nbits, "index out of bounds: {} >= {}", index, self.nbits);

        let bits = mem::size_of::<u64>() * 8;
        let w = index / bits;
        let b = index % bits;
        let flag = 1 << b;
        let val = if v {
            self.storage[w] | flag
        } else {
            self.storage[w] & !flag
        };

        self.storage[w] = val;
    }

    #[inline]
    pub fn iter(&self) -> Iter {
        Iter {
            array: self,
            range: 0..self.nbits,
        }
    }
}

pub struct Iter<'a> {
    array: &'a BitArray,
    range: Range<usize>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<bool> {
        self.range.next().map(|i| self.array.get(i).unwrap())
    }
}