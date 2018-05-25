use wasm::VmCtx;
use object::{ProcessRef, CodeRef, HandleRights, HandleOffset};
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;

/// Create a process with the specified compiled code.
#[nebulet_abi]
pub fn process_create(code_handle: HandleOffset, vmctx: &VmCtx) -> Result<usize> {
    let process = &*vmctx.process;

    let handle_table = process.handle_table();

    let code_ref = {
        let handle_table = handle_table.read();

        let code_handle = handle_table.get(code_handle as usize)?;

        let code_rights = code_handle.rights();

        if !code_rights.contains(HandleRights::READ) {
            return Err(Error::ACCESS_DENIED);
        }

        // Try casting the handle to the correct type.
        // If this fails, return `Error::WRONG_TYPE`.
        code_handle.cast::<CodeRef>()?
    };

    let new_proc = ProcessRef::create(code_ref)?;

    // Return the index of the new process' handle
    // in the current process' handle table.
    {
        let mut handle_table = handle_table.write();

        let rights = HandleRights::READ | HandleRights::WRITE;

        handle_table.allocate(new_proc, rights)
    }
}

/// Start the supplied process.
#[nebulet_abi]
pub fn process_start(_proc_handle: HandleOffset, _vmctx: &VmCtx) -> Result<usize> {

    Ok(0)
}
