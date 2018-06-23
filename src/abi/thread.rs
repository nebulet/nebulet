use object::Thread;
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;
use wasm::UserData;

#[nebulet_abi]
pub fn thread_yield(_: &UserData) {
    Thread::yield_now();
}

#[nebulet_abi]
pub fn thread_spawn(func_table_index: u32, arg: u32, new_stack_offset: u32, user_data: &UserData) -> Result<u32> {
    let func_addr = {
        let table = user_data.instance.tables[0].write();
        *table
            .get(func_table_index as usize)
            .ok_or(Error::NOT_FOUND)?
            as *const ()
    };

    let code = user_data.process.code();

    let module_func_index = code
        .lookup_func_index(func_addr)
        .ok_or(Error::NOT_FOUND)?;
    
    let module = code.module();
    let sig_index = *module
        .functions
        .get(module.imported_funcs.len() + module_func_index)
        .ok_or(Error::NOT_FOUND)?;
    
    let signature = module
        .signatures
        .get(sig_index)
        .ok_or(Error::NOT_FOUND)?;

    use cretonne_codegen::ir::{types, ArgumentPurpose};
    
    if signature.params.len() == 2
        && signature.params[0].value_type == types::I32
        && signature.params[1].purpose == ArgumentPurpose::VMContext
        && signature.returns.len() == 0
    {
        // the signature is valid for threading

        let current_thread = Thread::current();
        let current_process = current_thread.parent();

        let handle = current_process.create_thread(func_addr, arg, new_stack_offset)?;
       
        Ok(handle.inner())
    } else {
        Err(Error::INVALID_ARG)
    }
}
