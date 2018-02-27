pub mod devices;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Encapsulate<T>(T);

impl<T> Encapsulate<T> {
    pub fn into(self) -> T {
        self.0
    }

    pub const fn from(value: T) -> Self {
        Encapsulate(value)
    }
}