//! Standalone runtime for WebAssembly using Cretonne. Provides functions to translate
//! `get_global`, `set_global`, `current_memory`, `grow_memory`, `call_indirect` that hardcode in
//! the translation the base addresses of regions of memory that will hold the globals, tables and
//! linear memories.
//!
//! Pretty much just taken from https://github.com/sunfishcode/wasmstandalone

pub mod module;
pub mod instance;
pub mod compilation;
#[macro_use]
mod abi_types;
mod abi;


pub use self::module::Module;
pub use self::compilation::{Compilation, Compiler};
pub use self::instance::{Instance, VmCtx};

use cretonne_wasm::{self, FunctionIndex, GlobalIndex, TableIndex, MemoryIndex, Global, Table, Memory,
                GlobalValue, SignatureIndex, FuncTranslator, WasmResult};
use cretonne_codegen::ir::{self, InstBuilder, FuncRef, ExtFuncData, ExternalName, Signature, AbiParam,
                   ArgumentPurpose, ArgumentLoc, ArgumentExtension, Function};
use cretonne_codegen::ir::types::*;
use cretonne_codegen::ir::condcodes::IntCC;
use cretonne_codegen::settings::CallConv;
use cretonne_codegen::cursor::FuncCursor;
use cretonne_codegen::{self, isa, settings, binemit};
use target_lexicon::{Triple, Architecture, Vendor, OperatingSystem, Environment, BinaryFormat, PointerWidth};
use wasmparser;

use nabi;

use alloc::{Vec, String};

/// Compute a `ir::ExternalName` for a given wasm function index.
pub fn get_func_name(func_index: FunctionIndex) -> cretonne_codegen::ir::ExternalName {
    debug_assert!(func_index as u32 as FunctionIndex == func_index);
    ir::ExternalName::user(0, func_index as u32)
}

/// An entity to export.
pub enum Export {
    /// Function export.
    Function(FunctionIndex),
    /// Table export.
    Table(TableIndex),
    /// Memory export.
    Memory(MemoryIndex),
    /// Global export.
    Global(GlobalIndex),
}

/// Implementation of a relocation sink that just saves all the information for later
pub struct RelocSink {
    // func: &'func ir::Function,
    /// Relocations recorded for the function.
    pub func_relocs: Vec<(Relocation, RelocationType)>,
}

impl binemit::RelocSink for RelocSink {
    fn reloc_ebb(
        &mut self,
        _offset: binemit::CodeOffset,
        _reloc: binemit::Reloc,
        _ebb_offset: binemit::CodeOffset,
    ) {
        // This should use the `offsets` field of `ir::Function`.
        unimplemented!();
    }
    fn reloc_external(
        &mut self,
        offset: binemit::CodeOffset,
        reloc: binemit::Reloc,
        name: &ExternalName,
        addend: binemit::Addend,
    ) {
        match *name {
            ExternalName::User {
                namespace: 0,
                index,
            } => {
                self.func_relocs.push(
                    (
                        Relocation {
                            reloc,
                            offset,
                            addend,
                        },
                        RelocationType::Normal(index as _),
                    )
                );
            },
            ExternalName::TestCase {
                length,
                ascii,
            } => {
                let (slice, _) = ascii.split_at(length as usize);
                let name = String::from_utf8(slice.to_vec()).unwrap();

                self.func_relocs.push(
                    (
                        Relocation {
                            reloc,
                            offset,
                            addend,
                        },
                        RelocationType::Intrinsic(name),
                    )
                );
            },
            _ => {
                unimplemented!();
            }
        }
    }
    fn reloc_jt(
        &mut self,
        _offset: binemit::CodeOffset,
        _reloc: binemit::Reloc,
        _jt: ir::JumpTable,
    ) {
        unimplemented!();
    }
}

impl RelocSink {
    fn new() -> RelocSink {
        RelocSink {
            func_relocs: Vec::new(),
        }
    }
}

/// A data initializer for linear memory.
#[derive(Debug)]
pub struct DataInitializer {
    /// The index of the memory to initialize.
    pub memory_index: MemoryIndex,
    /// Optionally a globalvar base to initialize at.
    pub base: Option<GlobalIndex>,
    /// A constant offset to initialize at.
    pub offset: usize,
    /// The initialization data.
    pub data: Vec<u8>,
}

/// References to the input wasm data buffer to be decoded and processed later.
/// separately from the main module translation.
pub struct LazyContents<'data> {
    /// References to the function bodies.
    pub function_body_inputs: Vec<&'data [u8]>,

    /// References to the data initializers.
    pub data_initializers: Vec<DataInitializer>,
}

impl<'data> LazyContents<'data> {
    fn new() -> Self {
        Self {
            function_body_inputs: Vec::new(),
            data_initializers: Vec::new(),
        }
    }
}

/// Object containing the standalone runtime information. To be passed after creation as argument
/// to `cton_wasm::translatemodule`.
pub struct ModuleEnvironment<'data, 'flags> {
    /// Compilation setting flags.
    pub flags: &'flags settings::Flags,

    /// Module information.
    pub module: Module,

    /// References to information to be decoded later.
    pub lazy: LazyContents<'data>,
}

impl<'data, 'flags> ModuleEnvironment<'data, 'flags> {
    /// Allocates the runtime data structures with the given isa.
    pub fn new(flags: &'flags settings::Flags, module: Module) -> Self {
        Self {
            flags,
            module,
            lazy: LazyContents::new(),
        }
    }

    fn func_env(&self) -> FuncEnvironment {
        FuncEnvironment::new(&self.flags, &self.module)
    }

    fn native_pointer(&self) -> ir::Type {
        use cretonne_wasm::FuncEnvironment;
        self.func_env().native_pointer()
    }

    /// Declare that translation of the module is complete. This consumes the
    /// `ModuleEnvironment` with its mutable reference to the `Module` and
    /// produces a `ModuleTranslation` with an immutable reference to the
    /// `Module`.
    pub fn finish_translation(self) -> ModuleTranslation<'data, 'flags> {
        ModuleTranslation {
            flags: self.flags,
            module: self.module,
            lazy: self.lazy,
        }
    }
}

/// The FuncEnvironment implementation for use by the `ModuleEnvironment`.
pub struct FuncEnvironment<'module_environment> {
    /// Compilation setting flags.
    settings_flags: &'module_environment settings::Flags,

    /// The module-level environment which this function-level environment belongs to.
    pub module: &'module_environment Module,

    /// The Cretonne global holding the base address of the memories vector.
    pub memories_base: Option<ir::GlobalVar>,

    /// The Cretonne global holding the base address of the globals vector.
    pub globals_base: Option<ir::GlobalVar>,

    /// The list of globals that hold table bases.
    pub tables_base: Option<ir::GlobalVar>,

    /// The external function declaration for implementing wasm's `current_memory`.
    pub current_memory_extfunc: Option<FuncRef>,

    /// The external function declaration for implementing wasm's `grow_memory`.
    pub grow_memory_extfunc: Option<FuncRef>,
    
    pub debug_addr_extfunc: Option<FuncRef>,
}

impl<'module_environment> FuncEnvironment<'module_environment> {
    fn new(
        flags: &'module_environment settings::Flags,
        module: &'module_environment Module,
    ) -> Self {
        Self {
            settings_flags: flags,
            module,
            memories_base: None,
            globals_base: None,
            tables_base: None,
            current_memory_extfunc: None,
            grow_memory_extfunc: None,
            debug_addr_extfunc: None,
        }
    }

    /// Transform the call argument list in preparation for making a call.
    /// This pushes the VMContext into the args list.
    fn get_real_call_args(func: &Function, call_args: &[ir::Value]) -> Vec<ir::Value> {
        let mut real_call_args = Vec::with_capacity(call_args.len() + 1);
        real_call_args.extend_from_slice(call_args);
        real_call_args.push(func.special_param(ArgumentPurpose::VMContext).unwrap());
        real_call_args
    }

    fn ptr_size(&self) -> usize {
        use cretonne_wasm::FuncEnvironment;
        if self.triple().pointer_width().unwrap() == PointerWidth::U64 {
            8
        } else {
            4
        }
    }

    fn get_table(&mut self, func: &mut ir::Function, table_index: TableIndex) -> ir::GlobalVar {
        let ptr_size = self.ptr_size();
        let base = self.tables_base.unwrap_or_else(|| {
            let offset = ((ptr_size as i32) * 2).into();
            let new_base = func.create_global_var(
                ir::GlobalVarData::VMContext { offset }
            );
            self.globals_base = Some(new_base);
            new_base
        });

        let offset = table_index as usize * ptr_size;
        let offset = (offset as i32).into();
        func.create_global_var(ir::GlobalVarData::Deref {
            base,
            offset,
        })
    }
}

impl<'module_environment> cretonne_wasm::FuncEnvironment for FuncEnvironment<'module_environment> {
    fn flags(&self) -> &settings::Flags {
        &self.settings_flags
    }

    fn triple(&self) -> &Triple {
        &Triple {
            architecture: Architecture::X86_64,
            vendor: Vendor::Unknown,
            operating_system: OperatingSystem::Nebulet,
            environment: Environment::Unknown,
            binary_format: BinaryFormat::Unknown,
        }
    }

    fn make_global(&mut self, func: &mut ir::Function, index: GlobalIndex) -> GlobalValue {
        // let ptr_size = self.ptr_size();
        let globals_base = self.globals_base.unwrap_or_else(|| {
            let new_base = func.create_global_var(
                ir::GlobalVarData::VMContext { offset: 0.into() },
            );
            self.globals_base = Some(new_base);
            new_base
        });
        let offset = index as usize * self.ptr_size();
        let gv = func.create_global_var(ir::GlobalVarData::Deref {
            base: globals_base,
            offset: (offset as i32).into(),
        });
        GlobalValue::Memory {
            gv,
            ty: self.module.globals[index].ty,
        }
    }

    fn make_heap(&mut self, func: &mut ir::Function, index: MemoryIndex) -> ir::Heap {
        use memory::WasmMemory;
        let ptr_size = self.ptr_size();
        let memories_base = self.memories_base.unwrap_or_else(|| {
            let new_base = func.create_global_var(ir::GlobalVarData::VMContext {
                offset: (ptr_size as i32).into(),
            });
            self.globals_base = Some(new_base);
            new_base
        });
        let offset = index as usize * ptr_size;
        let heap_base = func.create_global_var(ir::GlobalVarData::Deref {
            base: memories_base,
            offset: (offset as i32).into(),
        });
        let h = func.create_heap(ir::HeapData {
            base: ir::HeapBase::GlobalVar(heap_base),
            min_size: 0.into(),
            guard_size: (WasmMemory::DEFAULT_GUARD_SIZE as i64).into(),
            style: ir::HeapStyle::Static { bound: (WasmMemory::DEFAULT_HEAP_SIZE as i64).into() },
        });
        h
    }

    fn make_indirect_sig(&mut self, func: &mut ir::Function, index: SignatureIndex) -> ir::SigRef {
        func.import_signature(self.module.signatures[index].clone())
    }

    fn make_direct_func(&mut self, func: &mut ir::Function, index: FunctionIndex) -> ir::FuncRef {
        let sigidx = self.module.functions[index];
        let signature = func.import_signature(self.module.signatures[sigidx].clone());
        let name = get_func_name(index);
        // TODO(gmorenz): Can colocated be true?
        func.import_function(ir::ExtFuncData { name, signature, colocated: false })
    }

    fn translate_call_indirect(
        &mut self,
        mut pos: FuncCursor,
        table_index: TableIndex,
        _sig_index: SignatureIndex,
        sig_ref: ir::SigRef,
        callee: ir::Value,
        call_args: &[ir::Value],
    ) -> WasmResult<ir::Inst> {
        // TODO: Cretonne's call_indirect doesn't implement bounds checking
        // or signature checking, so we need to implement it ourselves.
        assert_eq!(table_index, 0, "non-default tables not supported yet");
        let table_gv = self.get_table(pos.func, table_index);
        let gv_addr = pos.ins().global_addr(self.native_pointer(), table_gv);

        let table_base = pos.ins().load(
            self.native_pointer(),
            ir::MemFlags::new(),
            gv_addr,
            0,
        );

        let table_len = pos.ins().load(
            I32,
            ir::MemFlags::new(),
            gv_addr,
            8 as i32,
        );

        let oob = pos.ins().icmp(
            IntCC::UnsignedGreaterThanOrEqual,
            callee,
            table_len,
        );

        pos.ins().trapnz(oob, ir::TrapCode::OutOfBounds);

        let entry_size = self.ptr_size() as i64;

        let callee = if self.native_pointer() != ir::types::I32 {
            pos.ins().uextend(self.native_pointer(), callee)
        } else {
            callee
        };

        let callee_scaled = pos.ins().imul_imm(callee, entry_size);
        
        let entry = pos.ins().iadd(table_base, callee_scaled);

        let callee_func = pos.ins().load(
            self.native_pointer(),
            ir::MemFlags::new(),
            entry,
            0,
        );

        pos.ins().trapz(
            callee_func,
            ir::TrapCode::IndirectCallToNull,
        );

        let real_call_args = FuncEnvironment::get_real_call_args(pos.func, call_args);
        Ok(pos.ins().call_indirect(sig_ref, callee_func, &real_call_args))
    }

    fn translate_call(
        &mut self,
        mut pos: FuncCursor,
        callee_index: FunctionIndex,
        callee: ir::FuncRef,
        call_args: &[ir::Value],
    ) -> WasmResult<ir::Inst> {
        let real_call_args = FuncEnvironment::get_real_call_args(pos.func, call_args);

        // Since imported functions are declared first,
        // this will be true if the callee is an imported function
        if callee_index < self.module.imported_funcs.len() { // external function
            let sig_ref = pos.func.dfg.ext_funcs[callee].signature;
            // convert callee into value needed for `call_indirect`
            let callee_value = pos.ins()
                .func_addr(self.native_pointer(), callee);

            Ok(pos.ins()
                .call_indirect(sig_ref, callee_value, &real_call_args))
        } else { // internal function
            Ok(pos.ins()
                .call(callee, &real_call_args))
        }
    }

    fn translate_grow_memory(
        &mut self,
        mut pos: FuncCursor,
        index: MemoryIndex,
        _heap: ir::Heap,
        val: ir::Value,
    ) -> WasmResult<ir::Value> {
        debug_assert_eq!(index, 0, "non-default memories not supported yet");
        let grow_mem_func = self.grow_memory_extfunc.unwrap_or_else(|| {
            let sig_ref = pos.func.import_signature(Signature {
                call_conv: CallConv::SystemV,
                argument_bytes: None,
                params: vec![AbiParam::new(I32), AbiParam::special(I64, ArgumentPurpose::VMContext)],
                returns: vec![AbiParam::new(I32)],
            });
            // FIXME: Use a real ExternalName system.
            // TODO(gmorenz): Can colocated be true?
            pos.func.import_function(ExtFuncData {
                name: ExternalName::testcase("grow_memory"),
                signature: sig_ref,
                colocated: false,
            })
        });

        self.grow_memory_extfunc = Some(grow_mem_func);

        let vmctx = pos.func.special_param(ArgumentPurpose::VMContext).unwrap();

        let call_inst = pos.ins().call(grow_mem_func, &[val, vmctx]);
        Ok(*pos.func.dfg.inst_results(call_inst).first().unwrap())
    }

    fn translate_current_memory(
        &mut self,
        mut pos: FuncCursor,
        index: MemoryIndex,
        _heap: ir::Heap,
    ) -> WasmResult<ir::Value> {
        debug_assert_eq!(index, 0, "non-default memories not supported yet");
        let cur_mem_func = self.current_memory_extfunc.unwrap_or_else(|| {
            let sig_ref = pos.func.import_signature(Signature {
                call_conv: CallConv::SystemV,
                argument_bytes: None,
                params: vec![AbiParam::special(I64, ArgumentPurpose::VMContext)],
                returns: vec![AbiParam::new(I32)],
            });
            // FIXME: Use a real ExternalName system.
            // TODO(gmorenz): Can colocated be true?
            pos.func.import_function(ExtFuncData {
                name: ExternalName::testcase("current_memory"),
                signature: sig_ref,
                colocated: false,
            })
        });

        self.current_memory_extfunc = Some(cur_mem_func);

        let vmctx = pos.func.special_param(ArgumentPurpose::VMContext).unwrap();

        let call_inst = pos.ins().call(cur_mem_func, &[vmctx]);
        Ok(*pos.func.dfg.inst_results(call_inst).first().unwrap())
    }
}

/// This trait is useful for
/// `cton_wasm::translatemodule` because it
/// tells how to translate runtime-dependent wasm instructions. These functions should not be
/// called by the user.
impl<'data, 'flags> cretonne_wasm::ModuleEnvironment<'data> for ModuleEnvironment<'data, 'flags> {
    fn get_func_name(&self, func_index: FunctionIndex) -> cretonne_codegen::ir::ExternalName {
        get_func_name(func_index)
    }

    fn flags(&self) -> &settings::Flags {
        self.flags
    }

    fn declare_signature(&mut self, sig: &ir::Signature) {
        let mut sig = sig.clone();
        sig.params.push(AbiParam {
            value_type: self.native_pointer(),
            purpose: ArgumentPurpose::VMContext,
            extension: ArgumentExtension::None,
            location: ArgumentLoc::Unassigned,
        });
        // TODO: Deduplicate signatures.
        self.module.signatures.push(sig);
    }

    fn get_signature(&self, sig_index: SignatureIndex) -> &ir::Signature {
        &self.module.signatures[sig_index]
    }

    fn declare_func_import(&mut self, sig_index: SignatureIndex, module: &str, field: &str) {
        debug_assert_eq!(
            self.module.functions.len(),
            self.module.imported_funcs.len(),
            "Imported functions must be declared first"
        );
        self.module.functions.push(sig_index);

        self.module.imported_funcs.push((
            String::from(module),
            String::from(field),
        ));
    }

    fn get_num_func_imports(&self) -> usize {
        self.module.imported_funcs.len()
    }

    fn declare_func_type(&mut self, sig_index: SignatureIndex) {
        self.module.functions.push(sig_index);
    }

    fn get_func_type(&self, func_index: FunctionIndex) -> SignatureIndex {
        self.module.functions[func_index]
    }

    fn declare_global(&mut self, global: Global) {
        self.module.globals.push(global);
    }

    fn get_global(&self, global_index: GlobalIndex) -> &cretonne_wasm::Global {
        &self.module.globals[global_index]
    }

    fn declare_table(&mut self, table: Table) {
        self.module.tables.push(table);
    }

    fn declare_table_elements(
        &mut self,
        table_index: TableIndex,
        base: Option<GlobalIndex>,
        offset: usize,
        elements: Vec<FunctionIndex>,
    ) {
        debug_assert!(base.is_none(), "global-value offsets not supported yet");
        self.module.table_elements.push(module::TableElements {
            table_index,
            base,
            offset,
            elements,
        });
    }

    fn declare_memory(&mut self, memory: Memory) {
        self.module.memories.push(memory);
    }

    fn declare_data_initialization(
        &mut self,
        memory_index: MemoryIndex,
        base: Option<GlobalIndex>,
        offset: usize,
        data: &'data [u8],
    ) {
        debug_assert!(base.is_none(), "global-value offsets not supported yet");
        self.lazy.data_initializers.push(DataInitializer {
            memory_index,
            base,
            offset,
            data: data.to_vec(),
        });
    }

    fn declare_func_export(&mut self, func_index: FunctionIndex, name: &str) {
        self.module.exports.insert(
            String::from(name),
            module::Export::Function(func_index),
        );
    }

    fn declare_table_export(&mut self, table_index: TableIndex, name: &str) {
        self.module.exports.insert(
            String::from(name),
            module::Export::Table(table_index),
        );
    }

    fn declare_memory_export(&mut self, memory_index: MemoryIndex, name: &str) {
        self.module.exports.insert(
            String::from(name),
            module::Export::Memory(memory_index),
        );
    }

    fn declare_global_export(&mut self, global_index: GlobalIndex, name: &str) {
        self.module.exports.insert(
            String::from(name),
            module::Export::Global(global_index),
        );
    }

    fn declare_start_func(&mut self, func_index: FunctionIndex) {
        debug_assert!(self.module.start_func.is_none());
        self.module.start_func = Some(func_index);
    }

    fn define_function_body(&mut self, body_bytes: &'data [u8]) -> WasmResult<()> {
        self.lazy.function_body_inputs.push(body_bytes);
        Ok(())
    }
}

/// A record of a relocation to perform.
#[derive(Debug)]
pub struct Relocation {
    /// The relocation code.
    pub reloc: binemit::Reloc,
    /// The offset where to apply the relocation.
    pub offset: binemit::CodeOffset,
    /// The addend to add to the relocation value.
    pub addend: binemit::Addend,
}

/// Specify the type of relocation
#[derive(Debug)]
pub enum RelocationType {
    Normal(FunctionIndex),
    Intrinsic(String),
}

/// Relocations to apply to function bodies.
pub type Relocations = Vec<Vec<(Relocation, RelocationType)>>;

/// The result of translating via `ModuleEnvironment`.
pub struct ModuleTranslation<'data, 'flags> {
    /// Compilation setting flags.
    pub flags: &'flags settings::Flags,

    /// Module information.
    pub module: Module,

    /// Pointers into the raw data buffer.
    pub lazy: LazyContents<'data>,
}

/// Convenience functions for the user to be called after execution for debug purposes.
impl<'data, 'flags> ModuleTranslation<'data, 'flags> {
    fn func_env(&self) -> FuncEnvironment {
        FuncEnvironment::new(&self.flags, &self.module)
    }

    /// Compile the module, producing a compilation result with associated
    /// relocations.
    pub fn compile(
        self,
        isa: &isa::TargetIsa,
    ) -> Result<(Compilation, Module, Vec<DataInitializer>), nabi::Error> {
        let mut compiler = Compiler::with_capacity(isa, self.lazy.function_body_inputs.len());

        for (func_index, input) in self.lazy.function_body_inputs.iter().enumerate() {
            let mut context = cretonne_codegen::Context::new();
            context.func.name = get_func_name(func_index);
            let num_imported = self.module.imported_funcs.len();
            context.func.signature = self.module.signatures[self.module.functions[num_imported + func_index]].clone();

            let mut trans = FuncTranslator::new();
            let reader = wasmparser::BinaryReader::new(input);
            trans.translate_from_reader(reader, &mut context.func, &mut self.func_env())
                .map_err(|_| nabi::internal_error!())?;

            compiler.define_function(context)?;
        }

        let compilation = compiler.compile(&self.module)?;

        Ok((compilation, self.module, self.lazy.data_initializers))
    }
}
