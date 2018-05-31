use abi;
pub use super::abi_types::AbiFunction;

// TODO: Verify function signatures so we don't
// throw bad data at functions and crash everything.
abi_map! {
    // testing
    exit: { // eventually will exit maybe, right now is just for testing
        params: [I64],
        returns: I64,
        abi::test::output_test,
    },
    write_i64: {
        params: [I32, I64],
        returns: I64,
        abi::test::write_i64,
    },
    assert_eq: {
        params: [I64, I64],
        returns: I64,
        abi::test::assert_eq,
    },
    // intrinsics
    grow_memory: {
        params: [I32],
        returns: I32,
        abi::intrinsics::grow_memory,
    },
    current_memory: {
        params: [],
        returns: I32,
        abi::intrinsics::current_memory,
    },
    // actual abis
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
