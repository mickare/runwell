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

//! Runwell IR data structures, SSA builder and IR interpreter.

pub mod builder;
mod error;
mod instruction;
pub mod primitive;
mod store;

use self::builder::Variable;
pub use self::{
    builder::FunctionBuilderError,
    error::IrError,
    instruction::Alignment,
    store::Store,
};

/// All Runwell IR SSA instructions.
pub mod instr {
    /// The operands for some of the instructions.
    pub mod operands {
        #[doc(inline)]
        pub use super::super::instruction::{
            BinaryFloatOp,
            BinaryIntOp,
            CompareFloatOp,
            CompareIntOp,
            UnaryFloatOp,
            UnaryIntOp,
        };
    }
    #[doc(inline)]
    pub use super::instruction::{
        BinaryFloatInstr,
        BinaryIntInstr,
        BranchInstr,
        BranchTableInstr,
        CallIndirectInstr,
        CallInstr,
        CompareFloatInstr,
        CompareIntInstr,
        ConstInstr,
        DemoteFloatInstr,
        ExtendIntInstr,
        FloatInstr,
        FloatToIntInstr,
        IfThenElseInstr,
        Instruction,
        IntInstr,
        IntToFloatInstr,
        LoadInstr,
        MemoryGrowInstr,
        MemorySizeInstr,
        PhiInstr,
        PromoteFloatInstr,
        ReinterpretInstr,
        ReturnInstr,
        SelectInstr,
        StoreInstr,
        TailCallInstr,
        TerminalInstr,
        TruncateIntInstr,
        UnaryFloatInstr,
        UnaryIntInstr,
    };
}
