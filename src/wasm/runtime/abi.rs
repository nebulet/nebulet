use hashmap_core::HashMap;

use abi;

macro_rules! abi_map {
    ( $($name:ident: $func:path,)* ) => {
        lazy_static! {
            pub static ref ABI_MAP: HashMap<&'static str, usize> = {
                let mut m = HashMap::new();
                $(
                    m.insert(stringify!($name), $func as _);
                )*
                m
            };
        }
    };
    ( $($name:ident: $func:path),* ) => {
        abi_map!{$($name: $func,)*}
    };
}

abi_map! {
    exit: abi::output_test,
}