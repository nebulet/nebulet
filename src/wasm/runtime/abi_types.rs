use cretonne::ir::{ArgumentPurpose, Type, Signature, CallConv};

#[derive(Debug)]
pub struct AbiFunction {
    pub params: &'static [Type],
    pub returns: Type,
    pub ptr: *const u8,
}

impl AbiFunction {
    /// Check if the supplied signature has the same signature as `self`.
    /// TODO: Rewrite this to not be awful.
    pub fn same_sig(&self, sig: &Signature) -> bool {
        use cretonne::ir::types;
        if sig.call_conv != CallConv::Native {
            return false;
        }

        if sig.returns.len() > 1 {
            return false;
        }

        if self.returns == types::VOID && sig.returns.len() != 0 {
            return false;
        } else if sig.returns.len() == 1 && sig.returns[0].value_type != self.returns {
            return false;
        } else if sig.returns.len() > 1 {
            return false;
        }

        for i in 0..(sig.params.len() - 1) {
            if sig.params[i].value_type != self.params[i] {
                return false;
            }
        }

        if let Some(last_param) = sig.params.last() {
            if last_param.purpose != ArgumentPurpose::VMContext {
                return false;
            }
        } else {
            // no vmcontext
            return false;
        }

        return true;
    }
}

unsafe impl Sync for AbiFunction {}
unsafe impl Send for AbiFunction {}

macro_rules! abi_map {
    ( $($name:ident: { params: [ $($param:ident),* ], returns: $returns:ident, $func:path, }, )* ) => {
        use hashmap_core::HashMap;
        lazy_static! {
            pub static ref ABI_MAP: HashMap<&'static str, AbiFunction> = {
                use cretonne::ir::types::*;
                let mut m = HashMap::new();
                $(
                    m.insert(stringify!($name), AbiFunction {
                        params: &[$($param,)*],
                        returns: $returns,
                        ptr: $func as *const _,
                    });
                )*
                m
            };
        }
    };
}