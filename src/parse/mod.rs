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

//! Structures and routines for parsing and validating WebAssembly (Wasm).
//!
//! Use the [`parse`] function in order to parse and validate a Wasm encoded
//! stream of bytes.

mod error;
mod function;
mod global_variable;
mod id;
mod initializer;
mod module;
mod parser;
mod utils;

use self::module::ModuleBuilder;
pub use self::{
    error::ParseError,
    function::{Function, FunctionBody, FunctionSig},
    global_variable::{GlobalVariable, GlobalVariableDecl},
    id::{
        FunctionId,
        FunctionSigId,
        GlobalVariableId,
        Identifier,
        LinearMemoryId,
        TableId,
    },
    initializer::Initializer,
    module::{InternalFnIter, InternalGlobalIter, Module, Export},
    parser::parse,
};
