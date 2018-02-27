use alloc::{BTreeMap, Vec, String};
use alloc::boxed::Box;
use alloc::heap::Heap;
use alloc::allocator::{Alloc, Layout};
use alloc::arc::Arc;
use core::mem;
use spin::RwLock;

use context::{Context, ContextId, State, INITIAL_STACK_SIZE};

// List of Contexts
pub struct ContextList {
    map: BTreeMap<ContextId, Arc<RwLock<Context>>>,
    next: usize,
}

impl ContextList {
    pub fn new() -> ContextList {
        let mut context_map = BTreeMap::new();

        // The initial (kernel) process
        let mut kernel_context = Context::new(ContextId::KERNEL);
        kernel_context.state = State::Current;
        kernel_context.kstack = Some(vec![0; INITIAL_STACK_SIZE]);

        context_map.insert(ContextId::KERNEL, Arc::new(RwLock::new(kernel_context)));

        ContextList {
            map: context_map,
            next: 1,
        }
    }

    /// Create a new context
    pub fn create(&mut self) -> Result<&Arc<RwLock<Context>>, ()> {
        // Reset if we reach the max
        if self.next >= super::MAX_CONTEXTS {
            self.next = 1;
        }

        while self.map.contains_key(&ContextId::from(self.next)) {
            self.next += 1;
        }

        if self.next >= super::MAX_CONTEXTS {
            return Err(());
        }

        let id = ContextId::from(self.next);
        self.next += 1;

        assert!(self.map.insert(id, Arc::new(RwLock::new(Context::new(id)))).is_none());

        Ok(self.map.get(&id).expect("Failed to insert new context"))
    }

    /// Spawn a context from a function
    pub fn spawn(&mut self, f: extern "C" fn(), name: String) -> Result<ContextId, ()> {
        let context_lock = self.create()?;
        let mut context = context_lock.write();
        
        // Create an empty fx
        let mut fx = unsafe {
            Box::from_raw(Heap.alloc(Layout::from_size_align_unchecked(512, 16)).unwrap() as *mut [u8; 512])
        };
        for b in fx.iter_mut() {
            *b = 0;
        }

        // Create a stack (of 1KB initialially)
        let mut stack: Vec<u8> = vec![0; INITIAL_STACK_SIZE];
        let offset = stack.len() - mem::size_of::<usize>();
        // Place the function on top of the stack
        unsafe {
            let fn_ptr = stack.as_mut_ptr().offset(offset as isize);
            *(fn_ptr as *mut usize) = f as usize;
        }
        context.context.set_fx(fx.as_ptr() as usize);
        context.context.set_stack(stack.as_ptr() as usize + offset);
        context.kstack = Some(stack);
        context.name = Some(Arc::new(name.into_boxed_str()));

        Ok(context.id)
    }

    /// Retrive the given process
    pub fn get(&self, id: ContextId) -> Option<&Arc<RwLock<Context>>> {
        self.map.get(&id)
    }

    pub fn remove(&mut self, id: ContextId) -> Option<Arc<RwLock<Context>>> {
        self.map.remove(&id)
    }
}