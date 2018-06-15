use object::{Process, Wasm, ChannelRef, HandleRights, HandleOffset};
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

/// Create a process with the specified compiled code.
#[nebulet_abi]
pub fn process_create(code_handle: HandleOffset, channel_handle: HandleOffset, process: &Process) -> Result<u32> {
    let handle_table = process.handle_table();

    let (code_ref, chan_ref) = {
        let handle_table = handle_table.read();

        let code_handle = handle_table.get(code_handle as _)?;
        let chan_handle = handle_table.get(channel_handle as _)?;

        code_handle.check_rights(HandleRights::READ)?;
        chan_handle.check_rights(HandleRights::READ)?;

        // Try casting the handle to the correct type.
        // If this fails, return `Error::WRONG_TYPE`.
        (code_handle.cast::<Wasm>()?, chan_handle.cast::<ChannelRef>()?)
    };

    let new_proc = Process::create(code_ref)?;

    {
        let mut new_handle_table = new_proc.handle_table().write();
        let rights = HandleRights::READ;
        // this should set the 0th place in the handle table
        // of the new process as the handle to the read-end
        // of the supplied channel.
        let chan_index = new_handle_table.allocate(chan_ref, rights)?;
        assert_eq!(chan_index, 0);
    }

    // Return the index of the new process' handle
    // in the current process' handle table.
    {
        let mut handle_table = handle_table.write();

        let rights = HandleRights::READ | HandleRights::WRITE | HandleRights::TRANSFER;

        handle_table.allocate(new_proc, rights)
            .map(|handle| handle as u32)
    }
}

/// Start the supplied process.
#[nebulet_abi]
pub fn process_start(proc_handle: HandleOffset, process: &Process) -> Result<u32> {
    let handle_table = process.handle_table();

    let handle_table = handle_table.read();
    let proc_handle = handle_table.get(proc_handle as _)?;

    proc_handle.rights().has(HandleRights::WRITE)?;

    // Try casting the handle to the correct type.
    // If this fails, return `Error::WRONG_TYPE`.
    let proc_ref = proc_handle.cast::<Process>()?;

    proc_ref.start()?;

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
            .map(|handle| handle as u32)
    }
}
