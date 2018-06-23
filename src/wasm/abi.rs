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
    channel_write: {
        params: [I32, I32, I32],
        returns: I64,
        abi::ipc::channel_write,
    },
    channel_read: {
        params: [I32, I32, I32, I32],
        returns: I64,
        abi::ipc::channel_read,
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
    set_irq_handler: {
        params: [I32, I32],
        returns: VOID,
        abi::irq::set_irq_handler,
    },

    // events
    event_create: {
        params: [],
        returns: I64,
        abi::event::event_create,
    },
    event_wait: {
        params: [I32],
        returns: I64,
        abi::event::event_wait,
    },
    event_poll: {
        params: [I32],
        returns: I64,
        abi::event::event_poll,
    },
    event_trigger: {
        params: [I32],
        returns: I64,
        abi::event::event_trigger,
    },
    event_rearm: {
        params: [I32],
        returns: I64,
        abi::event::event_rearm,
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
