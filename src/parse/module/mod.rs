// Copyright 2020 Robin Freyler
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod builder;
mod data;
mod definitions;
mod iter;
mod structures;
mod table;

pub use self::{
    builder::{BuildError, ModuleBuilder},
    data::Data,
    definitions::{
        DefinedEntity,
        DefinedEntityMut,
        Entity,
        EntityIter,
        EntityMut,
        ImportName,
        ImportedEntity,
        ImportedEntityMut,
        ImportedOrDefined,
        ModuleError,
    },
    iter::InternalFnIter,
    structures::{Export, ExportKind},
    table::{Element, ElementItemsIter, TableDecl, TableItems},
};
use crate::parse::{
    utils::ImportedOrInternal,
    Function,
    FunctionBody,
    FunctionId,
    FunctionSig,
    FunctionSigId,
    GlobalInitExpr,
    GlobalVariableDecl,
    GlobalVariableId,
    Identifier,
    LinearMemoryId,
    TableId,
};
use wasmparser::MemoryType;

/// An iterator yielding global variables.
pub type GlobalVariableIter<'a> =
    EntityIter<'a, GlobalVariableId, GlobalVariableDecl, GlobalInitExpr>;

/// A parsed and validated WebAssembly (Wasm) module.
///
/// Use the [`parse`][`crate::parse::parse`] function in order to retrieve an instance of this type.
#[derive(Debug)]
pub struct Module {
    /// Function signature table.
    types: Vec<FunctionSig>,
    /// Imported and internal function signatures.
    fn_sigs: ImportedOrInternal<FunctionSigId, FunctionId>,
    /// Imported and internal global variables.
    globals:
        ImportedOrDefined<GlobalVariableId, GlobalVariableDecl, GlobalInitExpr>,
    /// Imported and internal linear memory sections.
    linear_memories: ImportedOrInternal<MemoryType, LinearMemoryId>,
    /// Imported and internal tables.
    tables: ImportedOrDefined<TableId, TableDecl, TableItems>,
    /// Export definitions.
    exports: Vec<Export>,
    /// Optional start function.
    ///
    /// # Note
    ///
    /// If this is `Some` the Wasm module is an executable,
    /// otherwise it is a library.
    start_fn: Option<FunctionId>,
    /// Internal function bodies.
    fn_bodies: Vec<FunctionBody>,
    /// Generic data of the Wasm module.
    ///
    /// # Note
    ///
    /// Used to initialize the linear memory section.
    data: Vec<Data>,
}

/// The kind of an entity that can be imported or defined internally.
#[derive(Debug, Copy, Clone)]
pub enum ImportExportKind {
    /// A function.
    Function,
    /// A global variable.
    Global,
    /// A table.
    Table,
    /// A linear memory.
    LinearMemory,
}

impl<'a> Module {
    /// Returns the number of imported items from the given kind.
    pub fn len_imported(&self, kind: ImportExportKind) -> usize {
        match kind {
            ImportExportKind::Function => self.fn_sigs.len_imported(),
            ImportExportKind::Global => self.globals.len_imported(),
            ImportExportKind::Table => self.tables.len_imported(),
            ImportExportKind::LinearMemory => {
                self.linear_memories.len_imported()
            }
        }
    }

    /// Returns the number of internal items from the given kind.
    pub fn len_internal(&self, kind: ImportExportKind) -> usize {
        match kind {
            ImportExportKind::Function => self.fn_sigs.len_internal(),
            ImportExportKind::Global => self.globals.len_defined(),
            ImportExportKind::Table => self.tables.len_defined(),
            ImportExportKind::LinearMemory => {
                self.linear_memories.len_internal()
            }
        }
    }

    /// Returns the function signature identified by `id`.
    fn get_signature(&self, id: FunctionSigId) -> &FunctionSig {
        &self.types[id.get()]
    }

    /// Returns the function identified by `id`.
    pub fn get_fn(&self, id: FunctionId) -> Function {
        let fn_sig = self.get_signature(self.fn_sigs[id]);
        Function::new(id, fn_sig)
    }

    /// Returns the function body identified by `id`.
    ///
    /// Returns `None` if the identified function is imported.
    ///
    /// # Note
    ///
    /// Required for utilities such as `start_fn`.
    pub fn get_fn_body(&self, id: FunctionId) -> Option<&FunctionBody> {
        id.get()
            // Convert the identified into an internal one.
            .checked_sub(self.fn_sigs.len_imported())
            .map(|internal_id| &self.fn_bodies[internal_id])
    }

    /// Returns the global variable identified by `id`.
    pub fn get_global(
        &self,
        id: GlobalVariableId,
    ) -> Entity<GlobalVariableId, GlobalVariableDecl, GlobalInitExpr> {
        self.globals
            .get(id)
            .expect("encountered unexpected invalid global variable ID")
    }

    /// Returns the linear memory identified by `id`.
    ///
    /// # Note
    ///
    /// Operations in Wasm that do not specify a linear memory ID explicitely
    /// are implicitely refering to the linear memory that is identified by
    /// the `0` ID. Use the
    /// [`Default`](https://doc.rust-lang.org/core/default/trait.Default.html)
    /// implementation of
    /// [`LinearMemoryId`][`crate::parse::LinearMemoryId`] in order to receive
    /// the implicit linear memory.
    ///
    /// ```no_run
    /// # let module: runwell::parse::Module = unimplemented!();
    /// let mem = module.get_linear_memory(Default::default());
    /// ```
    pub fn get_linear_memory(&self, id: LinearMemoryId) -> &MemoryType {
        &self.linear_memories[id]
    }

    /// Returns the table identified by `id`.
    ///
    /// # Note
    ///
    /// Operations in Wasm that do not specify a table ID explicitely
    /// are implicitely refering to the table that is identified by
    /// the `0` ID. Use the
    /// [`Default`](https://doc.rust-lang.org/core/default/trait.Default.html)
    /// implementation of
    /// [`TableId`][`crate::parse::TableId`] in order to receive
    /// the implicit table.
    ///
    /// ```no_run
    /// # let module: runwell::parse::Module = unimplemented!();
    /// let table = module.get_table(Default::default());
    /// ```
    pub fn get_table(
        &self,
        id: TableId,
    ) -> Entity<TableId, TableDecl, TableItems> {
        self.tables
            .get(id)
            .expect("encountered unexpected invalid table ID")
    }

    /// Returns an iterator over all internal functions and their bodies.
    pub fn iter_internal_fns(&self) -> InternalFnIter {
        InternalFnIter::new(self)
    }

    /// Returns an iterator over all imported or defined global variables.
    pub fn iter_globals(&self) -> GlobalVariableIter {
        self.globals.iter().expect(
            "encountered unexpected error upon iterating global variables",
        )
    }

    /// Returns an iterator over the exports of the Wasm module.
    pub fn iter_exports(&self) -> core::slice::Iter<Export> {
        self.exports.iter()
    }

    /// Returns the start function of the Wasm module if any.
    pub fn start_fn(&self) -> Option<Function> {
        self.start_fn.map(|fn_id| self.get_fn(fn_id))
    }
}

impl<'a> Module {
    /// Creates a new empty Wasm module.
    fn new() -> Self {
        Self {
            types: Vec::new(),
            fn_sigs: ImportedOrInternal::new(),
            globals: ImportedOrDefined::default(),
            linear_memories: ImportedOrInternal::new(),
            tables: ImportedOrDefined::default(),
            exports: Vec::new(),
            start_fn: None,
            fn_bodies: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Helps to build up a new Wasm module.
    pub(super) fn build() -> ModuleBuilder {
        ModuleBuilder::new(Self::new())
    }
}
