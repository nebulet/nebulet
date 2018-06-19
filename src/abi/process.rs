use object::{Process, Wasm, Channel, HandleRights, UserHandle};
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

/// Create a process with the specified compiled code.
#[nebulet_abi]
pub fn process_create(code_handle: UserHandle<Wasm>, channel_handle: UserHandle<Channel>, process: &Process) -> Result<u32> {
    let handle_table = process.handle_table();

    let (code, chan) = {
        let handle_table = handle_table.read();

        let code_handle = handle_table.get(code_handle)?;
        let chan_handle = handle_table.get(channel_handle)?;

        code_handle.check_rights(HandleRights::READ)?;
        chan_handle.check_rights(HandleRights::READ)?;

        // Try casting the handle to the correct type.
        // If this fails, return `Error::WRONG_TYPE`.
        (code_handle, chan_handle)
    };

    let new_proc = Process::create(code.refptr())?;

    {
        let mut new_handle_table = new_proc.handle_table().write();
        let rights = HandleRights::READ;
        // this should set the 0th place in the handle table
        // of the new process as the handle to the read-end
        // of the supplied channel.
        let chan_handle = new_handle_table.allocate(chan.refptr(), rights)?;
        assert_eq!(chan_handle.inner(), 0);
    }

    // Return the index of the new process' handle
    // in the current process' handle table.
    {
        let mut handle_table = handle_table.write();

        let rights = HandleRights::READ | HandleRights::WRITE | HandleRights::TRANSFER;

        handle_table.allocate(new_proc, rights)
            .map(|handle| handle.inner())
    }
}

/// Start the supplied process.
#[nebulet_abi]
pub fn process_start(proc_handle: UserHandle<Process>, process: &Process) -> Result<u32> {
    let handle_table = process.handle_table();

    let handle_table = handle_table.read();
    let proc_ref = handle_table.get(proc_handle)?;

    proc_ref
        .check_rights(HandleRights::WRITE)?
        .start()?;

    Ok(0)
}

/// Compile wasm bytecode into a Wasm.
#[nebulet_abi]
pub fn wasm_compile(buffer_offset: u32, buffer_size: u32, process: &Process) -> Result<u32> {
    let code_ref = {
        let instance = process.instance().read();
        let wasm_memory = &instance.memories[0];
        let wasm_bytecode = wasm_memory.carve_slice(buffer_offset, buffer_size)
            .ok_or(Error::INVALID_ARG)?;

        Wasm::compile(wasm_bytecode)?
    };

    {
        let mut handle_table = process.handle_table().write();
        let rights = HandleRights::READ | HandleRights::TRANSFER;

        handle_table.allocate(code_ref, rights)
            .map(|handle| handle.inner())
    }
}
