use core::fmt;

pub mod devices;
pub mod bitarray;
pub mod table;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub struct Encapsulate<T>(T);

impl<T> Encapsulate<T> {
    pub fn into(self) -> T {
        self.0
    }

    pub const fn from(value: T) -> Self {
        Encapsulate(value)
    }
}

impl<T: fmt::Display> fmt::Display for Encapsulate<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}