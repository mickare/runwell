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

use crate::{
    ir::{BasicBlockId, IrError, Type, Value},
    Index32,
};
use derive_more::{Display, From};
use std::collections::{hash_map::Entry, HashMap};
use crate::ir::{FunctionBuilderError, builder::VariableAccess};

define_id_type! {
    /// Represents a unique variable from the input language.
    ///
    /// Used to translate a foreign language into SSA form.
    ///
    /// # Note
    ///
    /// In the context of Wasm such variables are local variables that can
    /// be operated on using `local.set`, `local.get` and `local.tee`. Those
    /// operations are not in SSA form and we use the `Variable` index type
    /// in order to translate them to their SSA forms.
    ///
    /// # Example
    ///
    /// Since in Wasm all local variables in a function are uniquely identified
    /// by their local index we can simply take this local index and map it
    /// onto the `Variable` index space.
    #[derive(Display)]
    #[display(fmt = "var({})", "self.index.get()")]
    pub struct Variable;
}

/// Used to translate variables of some source language into Runwell IR SSA values.
///
/// All variables are required to be declared before their first use and they
/// are also required to be assigned to some value before they are read.
///
///
/// Upon first variable write the declarations array is traversed using binary
/// search and the associated declaration is inserted into the variable definitions
/// map for faster query upon the next time the same variable is written to again.
///
/// # Execution Time
///
/// ## Variable Declarations
///
/// All variable declarations have a constant execution time of O(1).
///
/// ## Variable Writes
///
/// The first time a variable is assigned that has been declared with a shared
/// declaration the `var_to_type` array is traversed using binary search taking
/// roughly O(log(D)) where D is the number of shared variable declarations.
/// Due to caching this occures only once per unique variable assignment.
/// Therefore the worst-case is triggered only whenever a shared declared variable
/// is only ever assigned to a new value once in the entire function.
/// The total worst-case execution time is O(A * log(D)) where A is the number of
/// unique variable assignments and D is the number of shared variable declarations.
///
/// ## Variable Reads
///
/// Both [`read_var`] as well as [`VariableDefinitions::for_block`] have a constant
/// execution time of O(1). However, reading the value of a variable during translation
/// might call [`VariableDefinitions::for_block`] multiple times for each recursive
/// predecessor of the current basic block. Therefore the execution time of reading
/// a variable is in O(P) where P is the set of predecessors of the current basic block
/// in the worst case.
///
/// # Dev. Note
///
/// As stated above the total worst-case execution time for variable assignments is in
/// O(A * log(D)) where A is the number of unique variable assignments and D is the number
/// of shared variable declarations.
/// In typical Wasm binaries D is very small leading to linear translation time.
/// Due to caching if A and D are equal the execution time is only O(A).
/// The worst case is if D is equal to A/2 with a worst-case execution time of
/// O(A * log(A/2)). The worst-case can be easily eliminated by requiring that types of variable
/// declarations in a function are required to be unique. As stated above this is already
/// true for typical generated Wasm binaries, e.g. in case of LLVM translations.
#[derive(Debug)]
pub struct VariableTranslator {
    /// The amount of declared variables.
    len_vars: u32,
    /// For every declaration of multiple variables their shared declaration is appended
    /// to this vector.
    ///
    /// # Note
    ///
    /// Upon first variable write the declarations array is traversed using binary
    /// search and the associated declaration is inserted into the variable definitions
    /// map for faster query upon the next time the same variable is written to again.
    var_to_type: Vec<VariableDecl>,
    /// Entries for variables definitions and their declared types.
    ///
    /// # Note
    ///
    /// This map is initialized lazily during the first assignment of each variable.
    var_to_defs: HashMap<Variable, VariableDefs>,
}

/// Space efficient storage for variable declarations and their declared types.
///
/// Used for storing shared information about variables that have been declared
/// together using [`VariableTranslator::declare_variables`] for more than just
/// a single variable.
#[derive(Debug)]
struct VariableDecl {
    /// Denotes the first variable index of the variable declarations that share
    /// the same type. All those declared variables have adjacent indices.
    offset: u32,
    /// The shared type of the variable declaration.
    ty: Type,
}

/// The entry for the definitions and the type of a declared variable.
///
/// Stores all definitions for all basic blocks for the variable
/// as well as the type of the variable's declaration.
#[derive(Debug)]
struct VariableDefs {
    /// All definitions for the variable per basic block.
    defs: HashMap<BasicBlockId, Value>,
    /// The type of the variable given upon its declaration.
    ty: Type,
}

impl VariableDefs {
    /// Create a new entry for variable definitions.
    pub fn new(ty: Type) -> Self {
        Self {
            defs: HashMap::new(),
            ty,
        }
    }
}

/// The value definitions of a variable for every basic block.
#[derive(Debug, Copy, Clone, From)]
pub struct VariableDefinitions<'a> {
    defs: &'a HashMap<BasicBlockId, Value>,
}

impl<'a> VariableDefinitions<'a> {
    /// Returns the value written to the variable for the given block if any.
    pub fn for_block(self, block: BasicBlockId) -> Option<Value> {
        self.defs.get(&block).copied()
    }
}

impl VariableTranslator {
    /// Returns the number of declared variables.
    fn len_vars(&self) -> usize {
        self.len_vars as usize
    }

    /// Returns `true` if the variable has been declared.
    fn is_declared(&self, var: Variable) -> bool {
        var.into_u32() < self.len_vars
    }

    /// Ensures that the variable has been declared.
    ///
    /// # Errors
    ///
    /// If the variable has not been declared.
    fn ensure_declared(
        &self,
        var: Variable,
        access: VariableAccess,
    ) -> Result<(), IrError> {
        if !self.is_declared(var) {
            return Err(FunctionBuilderError::MissingDeclarationForVariable {
                variable: var,
                access,
            })
            .map_err(Into::into)
        }
        Ok(())
    }

    /// Ensures that the type of the variable declaration matches the type of the new value.
    ///
    /// # Errors
    ///
    /// If the type of the new value does not match the type of the variable declaration.
    fn ensure_types_match<F>(
        var: Variable,
        new_value: Value,
        declared_type: Type,
        value_to_type: F,
    ) -> Result<(), IrError>
    where
        F: FnOnce(Value) -> Type,
    {
        let value_type = value_to_type(new_value);
        if declared_type != value_type {
            return Err(FunctionBuilderError::UnmatchingVariableType {
                variable: var,
                value: new_value,
                declared_type,
                value_type,
            })
            .map_err(Into::into)
        }
        Ok(())
    }

    /// Declares an amount of variables that share the same type.
    ///
    /// A variable is required to be declared before reading or writing to it
    /// using [`VariableTranslator::read_var`] and [`VariableTranslator::write_var`].
    ///
    /// # Errors
    ///
    /// If there are more than 2^31 variable declarations.
    pub fn declare_vars(
        &mut self,
        amount: u32,
        ty: Type,
    ) -> Result<(), IrError> {
        let offset = self.len_vars;
        self.len_vars += amount;
        if self.len_vars >= u32::MAX {
            return Err(FunctionBuilderError::TooManyVariableDeclarations)
                .map_err(Into::into)
        }
        self.var_to_type.push(VariableDecl { offset, ty }); // TODO: maybe we can get rid of this if amount == 1
        if amount == 1 {
            // As an optimization we directly initialize the definition of the
            // variable to avoid the binary search for it upon its first assignmnet.
            let var = Variable::from_u32(offset);
            let old_def = self.var_to_defs.insert(var, VariableDefs::new(ty));
            debug_assert!(old_def.is_none());
        }
        Ok(())
    }

    /// Assigns a new value to the variable.
    ///
    /// - The variable assignment is with respect to the given basic block.
    /// - The `value_to_type` closure is used to check whether the type of the new value
    ///   matches the type given at variable declaration.
    ///
    /// # Errors
    ///
    /// - If the variable has not been declared before.
    /// - If the type of the new value does not match the type of the variable declaration.
    pub fn write_var<F>(
        &mut self,
        var: Variable,
        new_value: Value,
        block: BasicBlockId,
        value_to_type: F,
    ) -> Result<(), IrError>
    where
        F: FnOnce(Value) -> Type,
    {
        self.ensure_declared(var, VariableAccess::Write(new_value))?;
        let Self {
            var_to_type,
            var_to_defs,
            ..
        } = self;
        match var_to_defs.entry(var) {
            Entry::Occupied(occupied) => {
                // Variable has already been defined previously.
                // Check type of new assignment first and then update assignment.
                let declared_type = occupied.get().ty;
                Self::ensure_types_match(
                    var,
                    new_value,
                    declared_type,
                    value_to_type,
                )?;
                occupied.into_mut().defs.insert(block, new_value);
            }
            Entry::Vacant(vacant) => {
                // Variable has been declared but never been assigned before.
                // First figure out the type of the variable declaration,
                // then check if type of new assignment matches and finally
                // update the variable assignment.
                let target = var.into_u32();
                let declared_type = match var_to_type
                    .binary_search_by(|decl| target.cmp(&decl.offset))
                {
                    Ok(index) => var_to_type[index].ty,
                    Err(index) => var_to_type[index - 1].ty,
                };
                Self::ensure_types_match(
                    var,
                    new_value,
                    declared_type,
                    value_to_type,
                )?;
                vacant.insert(VariableDefs::new(declared_type));
            }
        }
        Ok(())
    }

    /// Returns all definitions per basic block of the variable.
    ///
    /// # Errors
    ///
    /// - If the variable has not been declared, yet.
    /// - If the variable has never been written to before.
    pub fn read_var(
        &self,
        var: Variable,
    ) -> Result<VariableDefinitions, IrError> {
        self.ensure_declared(var, VariableAccess::Read)?;
        self.var_to_defs
            .get(&var)
            .map(|entry| VariableDefinitions { defs: &entry.defs })
            .ok_or(FunctionBuilderError::ReadBeforeWriteVariable {
                variable: var,
            })
            .map_err(Into::into)
    }
}