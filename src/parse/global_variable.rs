// Copyright 2019 Robin Freyler
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

use wasmparser::{Type, Operator};
use crate::parse::GlobalVariableId;
use derive_more::From;
use core::iter::FromIterator;

/// A global variable declaration.
#[derive(Debug, From)]
pub struct GlobalVariableDecl {
    /// The `wasmparser` wrapped type.
    global_type: wasmparser::GlobalType,
}

impl GlobalVariableDecl {
    /// Returns the type of the global variable.
    pub fn ty(&self) -> &Type {
        &self.global_type.content_type
    }

    /// Returns `true` if `self` is mutable.
    pub fn is_mutable(&self) -> bool {
        self.global_type.mutable
    }
}

/// A global variable declaration.
#[derive(Debug)]
pub struct GlobalVariable {
    /// The global unique identifier of the global variable.
    id: GlobalVariableId,
    /// The declaration of the global variable.
    decl: GlobalVariableDecl,
}

impl GlobalVariable {
    /// Returns the identifier of the global variable.
    pub fn id(&self) -> GlobalVariableId {
        self.id
    }

    /// Returns the declaration of the global variable.
    pub fn decl(&self) -> &GlobalVariableDecl {
        &self.decl
    }
}

/// A global variable initializer code.
#[derive(Debug)]
pub struct GlobalVariableInitializer<'a> {
    /// The initializer code of a global variable.
    ops: Vec<Operator<'a>>,
}

impl<'a> GlobalVariableInitializer<'a> {
    /// Returns the operations of the initializer routine.
    pub fn ops(&self) -> &[Operator<'a>] {
        &self.ops
    }
}

impl<'a> FromIterator<Operator<'a>> for GlobalVariableInitializer<'a> {
    fn from_iter<T: IntoIterator<Item = Operator<'a>>>(iter: T) -> Self {
        Self { ops: iter.into_iter().collect() }
    }
}