// Copyright 2021 Robin Freyler
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

mod call;
mod constant;
mod conv;
mod float;
mod int;
mod memory;
mod phi;
mod select;
mod terminal;

pub use self::{
    call::{CallIndirectInstr, CallInstr},
    constant::ConstInstr,
    conv::ReinterpretInstr,
    float::{
        BinaryFloatInstr,
        BinaryFloatOp,
        CompareFloatInstr,
        CompareFloatOp,
        DemoteFloatInstr,
        FloatInstr,
        FloatToIntInstr,
        PromoteFloatInstr,
        UnaryFloatInstr,
        UnaryFloatOp,
    },
    int::{
        BinaryIntInstr,
        BinaryIntOp,
        CompareIntInstr,
        CompareIntOp,
        ExtendIntInstr,
        IntInstr,
        IntToFloatInstr,
        TruncateIntInstr,
        UnaryIntInstr,
        UnaryIntOp,
    },
    memory::{
        Alignment,
        LoadInstr,
        MemoryGrowInstr,
        MemorySizeInstr,
        StoreInstr,
    },
    phi::PhiInstr,
    select::SelectInstr,
    terminal::{
        BranchInstr,
        BranchTableInstr,
        IfThenElseInstr,
        ReturnInstr,
        TerminalInstr,
    },
};
use derive_more::{Display, From};

use super::{
    interpreter::{InterpretationContext, InterpretationError},
    primitive::Value,
};

/// An SSA instruction from the Runwell IR.
#[derive(Debug, Display, From, PartialEq, Eq, Hash)]
pub enum Instruction {
    Call(CallInstr),
    CallIndirect(CallIndirectInstr),
    Const(ConstInstr),
    MemoryGrow(MemoryGrowInstr),
    MemorySize(MemorySizeInstr),
    Phi(PhiInstr),
    Load(LoadInstr),
    Store(StoreInstr),
    Select(SelectInstr),
    Reinterpret(ReinterpretInstr),
    Terminal(TerminalInstr),
    Int(IntInstr),
    Float(FloatInstr),
}

impl Instruction {
    /// Returns `true` if the instruction terminates a basic block.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminal(_))
    }

    /// Returns `true` if the instruction is a ϕ-instruction.
    pub fn is_phi(&self) -> bool {
        matches!(self, Self::Phi(_))
    }

    /// Replaces all values in the instruction using the replacer.
    ///
    /// Returns `true` if a value has been replaced in the instruction.
    ///
    /// # Note
    ///
    /// By contract the replacer returns `true` if replacement happened.
    pub fn replace_value<F>(&mut self, replace: F) -> bool
    where
        F: FnMut(&mut Value) -> bool,
    {
        match self {
            Self::Call(instr) => instr.replace_value(replace),
            Self::CallIndirect(instr) => instr.replace_value(replace),
            Self::Const(_instr) => false,
            Self::MemoryGrow(instr) => instr.replace_value(replace),
            Self::MemorySize(_instr) => false,
            Self::Phi(instr) => instr.replace_value(replace),
            Self::Load(instr) => instr.replace_value(replace),
            Self::Store(instr) => instr.replace_value(replace),
            Self::Select(instr) => instr.replace_value(replace),
            Self::Reinterpret(instr) => instr.replace_value(replace),
            Self::Terminal(instr) => instr.replace_value(replace),
            Self::Int(instr) => instr.replace_value(replace),
            Self::Float(instr) => instr.replace_value(replace),
        }
    }

    /// Evaluates the function given the interpretation context.
    pub fn interpret(
        &self,
        value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        match self {
            Self::Call(_instr) => todo!(),
            Self::CallIndirect(_instr) => todo!(),
            Self::Const(instr) => instr.interpret(value, ctx),
            Self::MemoryGrow(_instr) => todo!(),
            Self::MemorySize(_instr) => todo!(),
            Self::Phi(instr) => instr.interpret(value, ctx),
            Self::Load(_instr) => todo!(),
            Self::Store(_instr) => todo!(),
            Self::Select(_instr) => todo!(),
            Self::Reinterpret(_instr) => todo!(),
            Self::Terminal(instr) => instr.interpret(value, ctx),
            Self::Int(instr) => instr.interpret(value, ctx),
            Self::Float(_instr) => todo!(),
        }
    }
}
