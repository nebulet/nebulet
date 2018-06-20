use object::{Process, Thread, HandleRights};
use nabi::{Result, Error};
use nebulet_derive::nebulet_abi;
use nil::Ref;

#[nebulet_abi]
pub fn thread_yield(_: &Process) {
    Thread::yield_now();
}

#[nebulet_abi]
pub fn thread_spawn(func_table_index: u32, arg: u32, process: &Ref<Process>) -> Result<u32> {
    let func_addr = {
        let table = &process.instance().read().tables;
        *table[0]
        .get(func_table_index as usize)
        .ok_or(Error::NOT_FOUND)?
    };

    let code = process.code();

    let module_func_index = code
        .lookup_func_index(func_addr as *const ())
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
        use core::mem::transmute;

        let f: extern fn(u32, &VmCtx) = unsafe { transmute(func_addr) };
        let current_thread = Thread::current();

        let parent_process = process.clone();
        let thread = Thread::new(current_thread.parent().clone(), 1024 * 1024, move || {
            let mut vmctx_gen = parent_process.instance().write().generate_vmctx_backing();
            let vmctx_ref = vmctx_gen.vmctx(parent_process);
            f(arg, vmctx_ref);
        })?;

        thread.resume();

        let mut handle_table = process.handle_table().write();

        let handle = handle_table.allocate(thread, HandleRights::READ | HandleRights::WRITE)?;

        Ok(handle.inner())
    } else {
        Err(Error::INVALID_ARG)
    }
}
