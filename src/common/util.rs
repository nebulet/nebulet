macro_rules! impl_parseable_num {
    ($t:ident) => {
        impl ParseableNum for $t {
            fn from_str_radix(s: &str, radix: u32) -> Option<$t> {
                $t::from_str_radix(s, radix).ok()
            }
        }
    };
    ($($t:ident),*) => {
        $(
            impl_parseable_num!($t);
        )*
    };
}

pub trait ParseableNum
    where Self: Sized
{
    fn from_str_radix(s: &str, radix: u32) -> Option<Self>;
}

impl_parseable_num!(usize, isize, u64, i64, u32, i32, u16, i16, u8, i8);

pub fn parse_num<T: ParseableNum>(mut s: &str) -> Option<T> {
    let radix = if s.starts_with("0x") {
        s = &s[2..];
        16
    } else if s.starts_with("0o") {
        s = &s[2..];
        8
    } else if s.starts_with("0b") {
        s = &s[2..];
        2
    } else {
        10
    };

    T::from_str_radix(s, radix)
}
