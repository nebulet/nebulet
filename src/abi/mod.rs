//! The interface between running processes and the kernel
//!

use wasm::runtime::instance::VmCtx;
use object::GlobalHandleTable;
use process::Process;

fn with_proc<F>(vmctx: &VmCtx, f: F)
    where F: FnOnce(&Process)
{
    let handle_table = GlobalHandleTable::get();
    {
        let process = handle_table
            .get_handle(vmctx.proc_index)
            .unwrap()
            .lock_cast::<Process>()
            .unwrap();
        
        f(&process);
    }
}

pub extern fn output_test(arg: usize, vmctx: &VmCtx) {
    println!("wasm supplied arg = {}", arg);
    
    with_proc(vmctx, |p| {
        println!("calling process name: \"{}\"", p.name);
    });
}
