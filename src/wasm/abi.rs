use abi;
pub use super::abi_types::AbiFunction;
use hashmap_core::HashMap;

// TODO: Verify function signatures so we don't
// throw bad data at functions and crash everything.
abi_map! {
    ABI_MAP,
    // testing
    exit: { // eventually will exit maybe, right now is just for testing
        params: [I64],
        returns: I64,
        abi::test::output_test,
    },

    // generic handle operations
    handle_close: {
        params: [I32],
        returns: I64,
        abi::handle::handle_close,
    },
    handle_duplicate: {
        params: [I32, I32],
        returns: I64,
        abi::handle::handle_duplicate,
    },

    // process
    wasm_compile: {
        params: [I32, I32],
        returns: I64,
        abi::process::wasm_compile,
    },
    process_create: {
        params: [I32, I32],
        returns: I64,
        abi::process::process_create,
    },
    process_start: {
        params: [I32],
        returns: I64,
        abi::process::process_start,
    },

    // ipc
    channel_create: {
        params: [I32, I32],
        returns: I64,
        abi::ipc::channel_create,
    },
    channel_send: {
        params: [I32, I32, I32],
        returns: I64,
        abi::ipc::channel_send,
    },
    channel_recv: {
        params: [I32, I32, I32, I32],
        returns: I64,
        abi::ipc::channel_recv,
    },

    stream_create: {
        params: [I32, I32],
        returns: I64,
        abi::ipc::stream_create,
    },
    stream_write: {
        params: [I32, I32, I32, I32],
        returns: I64,
        abi::ipc::stream_write,
    },
    stream_read: {
        params: [I32, I32, I32, I32],
        returns: I64,
        abi::ipc::stream_read,
    },

    // debug
    print: {
        params: [I32, I32],
        returns: VOID,
        abi::io::print,
    },

    // driver ABIs
    physical_map: {
        params: [I64, I32],
        returns: I64,
        abi::driver::physical_map,
    },
    read_port_u8: {
        params: [I32],
        returns: I32,
        abi::io::read_port_u8,
    },
    write_port_u8: {
        params: [I32, I32],
        returns: VOID,
        abi::io::write_port_u8,
    },
    create_irq_event: {
        params: [I32],
        returns: I64,
        abi::irq::create_irq_event,
    },
    ack_irq: {
        params: [I32],
        returns: I64,
        abi::irq::ack_irq,
    },

    // events
    event_create: {
        params: [],
        returns: I64,
        abi::event::event_create,
    },
    // objects
    object_wait_one: {
        params: [I32, I32],
        returns: I64,
        abi::object::object_wait_one,
    },
    object_signal: {
        params: [I32, I32, I32],
        returns: I64,
        abi::object::object_signal,
    },
    // threads
    thread_yield: {
        params: [],
        returns: VOID,
        abi::thread::thread_yield,
    },
    thread_spawn: {
        params: [I32, I32, I32],
        returns: I64,
        abi::thread::thread_spawn,
    },
    thread_join: {
        params: [I32],
        returns: I64,
        abi::thread::thread_join,
    },

    // Pretty fast exclusion
    pfex_acquire: {
        params: [I32],
        returns: VOID,
        abi::pfex::pfex_acquire,
    },
    pfex_release: {
        params: [I32],
        returns: VOID,
        abi::pfex::pfex_release,
    },
}

abi_map! {
    INTRINSIC_MAP,
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
    // debug_addr: {
    //     params: [I64],
    //     returns: VOID,
    //     abi::intrinsics::debug_addr,
    // },
}
