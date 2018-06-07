use object::{ProcessRef, CodeRef, HandleRights, HandleOffset};
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

/// Create a process with the specified compiled code.
#[nebulet_abi]
pub fn process_create(code_handle: HandleOffset, process: &ProcessRef) -> Result<u32> {
    let handle_table = process.handle_table();

    let code_ref = {
        let handle_table = handle_table.read();

        let code_handle = handle_table.get(code_handle as _)?;

        code_handle.rights().has(HandleRights::READ)?;

        // Try casting the handle to the correct type.
        // If this fails, return `Error::WRONG_TYPE`.
        code_handle.cast::<CodeRef>()?
    };

    let new_proc = ProcessRef::create(code_ref)?;

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
pub fn process_start(proc_handle: HandleOffset, process: &ProcessRef) -> Result<u32> {
    let handle_table = process.handle_table();

    let handle_table = handle_table.read();
    let proc_handle = handle_table.get(proc_handle as _)?;

    proc_handle.rights().has(HandleRights::WRITE)?;

    // Try casting the handle to the correct type.
    // If this fails, return `Error::WRONG_TYPE`.
    let proc_ref = proc_handle.cast::<ProcessRef>()?;

    proc_ref.start()?;

    Ok(0)
}

/// Compile wasm bytecode into a coderef.
#[nebulet_abi]
pub fn wasm_compile(buffer_offset: u32, buffer_size: u32, process: &ProcessRef) -> Result<u32> {
    let code_ref = {
        let instance = process.instance().read();
        let wasm_memory = &instance.memories[0];
        let wasm_bytecode = wasm_memory.carve_slice(buffer_offset, buffer_size)
            .ok_or(Error::INVALID_ARG)?;

        CodeRef::compile(wasm_bytecode)?
    };

    {
        let mut handle_table = process.handle_table().write();
        let rights = HandleRights::READ | HandleRights::TRANSFER;

        handle_table.allocate(code_ref, rights)
            .map(|handle| handle as u32)
    }
}
