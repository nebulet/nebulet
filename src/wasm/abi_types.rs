use cranelift_codegen::ir::{ArgumentPurpose, Signature, Type};
use cranelift_codegen::settings::CallConv;

#[derive(Debug)]
pub struct AbiFunction {
    pub params: &'static [Type],
    pub returns: Type,
    pub ptr: *const (),
}

impl AbiFunction {
    /// Check if the supplied signature has the same signature as `self`.
    /// TODO: Rewrite this to not be awful.
    pub fn same_sig(&self, sig: &Signature) -> bool {
        use cranelift_codegen::ir::types;
        if sig.call_conv != CallConv::SystemV {
            return false;
        }

        if sig.returns.len() > 1 {
            return false;
        } else if self.returns == types::VOID && sig.returns.len() != 0 {
            return false;
        } else if sig.returns.len() == 1 && sig.returns[0].value_type != self.returns {
            return false;
        } else if sig.returns.len() == 0 && self.returns != types::VOID {
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
    ( $map_name:ident, $($name:ident: { params: [ $($param:ident),* ], returns: $returns:ident, $func:path, }, )* ) => {
        lazy_static! {
            pub static ref $map_name: HashMap<&'static str, AbiFunction> = {
                use cranelift_codegen::ir::types::*;
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
