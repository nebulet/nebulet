use abi;
pub use super::abi_types::AbiFunction;

// TODO: Verify function signatures so we don't
// throw bad data at functions and crash everything.
abi_map! {
    exit: { // eventually will exit maybe, right now is just for testing
        params: [I64],
        returns: VOID, // indicates no return
        abi::output_test,
    },
}