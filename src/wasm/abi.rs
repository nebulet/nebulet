use abi;
pub use super::abi_types::AbiFunction;

// TODO: Verify function signatures so we don't
// throw bad data at functions and crash everything.
abi_map! {
    exit: { // eventually will exit maybe, right now is just for testing
        params: [I64],
        returns: I64,
        abi::test::output_test,
    },
    wasm_compile: {
        params: [I32, I32],
        returns: I64,
        abi::process::wasm_compile,
    },
    process_create: {
        params: [I32],
        returns: I64,
        abi::process::process_create,
    },
    process_start: {
        params: [I32],
        returns: I64,
        abi::process::process_start,
    },
}
