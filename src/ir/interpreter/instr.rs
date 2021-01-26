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

use super::{InterpretationContext, InterpretationError};
use crate::ir::{
    instr::{
        BinaryIntInstr,
        BranchInstr,
        CompareIntInstr,
        ConstInstr,
        ExtendIntInstr,
        IfThenElseInstr,
        Instruction,
        IntInstr,
        IntToFloatInstr,
        PhiInstr,
        ReinterpretInstr,
        ReturnInstr,
        SelectInstr,
        TerminalInstr,
        TruncateIntInstr,
        UnaryIntInstr,
    },
    instruction::{BinaryIntOp, CompareIntOp, UnaryIntOp},
    primitive::{IntType, Value},
};

/// Implemented by Runwell IR instructions to make them interpretable.
pub trait InterpretInstr {
    /// Evaluates the function given the interpretation context.
    fn interpret_instr(
        &self,
        return_return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError>;
}

impl InterpretInstr for Instruction {
    fn interpret_instr(
        &self,
        return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        match self {
            Self::Call(_instr) => unimplemented!(),
            Self::CallIndirect(_instr) => unimplemented!(),
            Self::Const(instr) => instr.interpret_instr(return_value, ctx),
            Self::MemoryGrow(_instr) => unimplemented!(),
            Self::MemorySize(_instr) => unimplemented!(),
            Self::Phi(instr) => instr.interpret_instr(return_value, ctx),
            Self::Load(_instr) => unimplemented!(),
            Self::Store(_instr) => unimplemented!(),
            Self::Select(instr) => instr.interpret_instr(return_value, ctx),
            Self::Reinterpret(instr) => {
                instr.interpret_instr(return_value, ctx)
            }
            Self::Terminal(instr) => instr.interpret_instr(return_value, ctx),
            Self::Int(instr) => instr.interpret_instr(return_value, ctx),
            Self::Float(_instr) => unimplemented!(),
        }
    }
}

impl InterpretInstr for PhiInstr {
    fn interpret_instr(
        &self,
        return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        let return_value = return_value.expect("missing value for instruction");
        let last_block = ctx
            .last_block()
            .expect("phi instruction is missing predecessor");
        let result = self
            .operand_for(last_block)
            .expect("phi instruction missing value for predecessor");
        let result = ctx.read_register(result);
        ctx.write_register(return_value, result);
        Ok(())
    }
}

impl InterpretInstr for ConstInstr {
    fn interpret_instr(
        &self,
        return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        let return_value = return_value.expect("missing value for instruction");
        ctx.write_register(return_value, self.const_value().into_bits64());
        Ok(())
    }
}

impl InterpretInstr for SelectInstr {
    fn interpret_instr(
        &self,
        return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        let return_value = return_value.expect("missing value for instruction");
        let condition = ctx.read_register(self.condition());
        let result_value = if condition != 0 {
            self.true_value()
        } else {
            self.false_value()
        };
        let result = ctx.read_register(result_value);
        ctx.write_register(return_value, result);
        Ok(())
    }
}

impl InterpretInstr for TerminalInstr {
    fn interpret_instr(
        &self,
        return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        match self {
            Self::Trap => {
                ctx.set_trapped();
                Ok(())
            }
            Self::Return(instr) => instr.interpret_instr(return_value, ctx),
            Self::Br(instr) => instr.interpret_instr(return_value, ctx),
            Self::Ite(instr) => instr.interpret_instr(return_value, ctx),
            Self::BranchTable(_instr) => unimplemented!(),
        }
    }
}

impl InterpretInstr for ReturnInstr {
    fn interpret_instr(
        &self,
        _return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        let return_value = ctx.read_register(self.return_value());
        ctx.set_return_value(&[return_value])?;
        Ok(())
    }
}

impl InterpretInstr for BranchInstr {
    fn interpret_instr(
        &self,
        _return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        ctx.switch_to_block(self.target());
        Ok(())
    }
}

impl InterpretInstr for IfThenElseInstr {
    fn interpret_instr(
        &self,
        _return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        let condition = ctx.read_register(self.condition());
        let target = if condition != 0 {
            self.true_target()
        } else {
            self.false_target()
        };
        ctx.switch_to_block(target);
        Ok(())
    }
}

impl InterpretInstr for ReinterpretInstr {
    fn interpret_instr(
        &self,
        return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        let return_value = return_value.expect("missing value for instruction");
        let source = ctx.read_register(self.src());
        debug_assert_eq!(
            self.src_type().bit_width(),
            self.dst_type().bit_width()
        );
        // Reinterpretation just moves from one register to the other.
        ctx.write_register(return_value, source);
        Ok(())
    }
}

impl InterpretInstr for IntInstr {
    fn interpret_instr(
        &self,
        return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        match self {
            Self::Binary(instr) => instr.interpret_instr(return_value, ctx),
            Self::Unary(instr) => instr.interpret_instr(return_value, ctx),
            Self::Compare(instr) => instr.interpret_instr(return_value, ctx),
            Self::Extend(instr) => instr.interpret_instr(return_value, ctx),
            Self::IntToFloat(instr) => instr.interpret_instr(return_value, ctx),
            Self::Truncate(instr) => instr.interpret_instr(return_value, ctx),
        }
    }
}

impl InterpretInstr for UnaryIntInstr {
    fn interpret_instr(
        &self,
        return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        let return_value = return_value.expect("missing value for instruction");
        let source = ctx.read_register(self.src());
        let result = match self.op() {
            UnaryIntOp::LeadingZeros => source.leading_zeros(),
            UnaryIntOp::TrailingZeros => source.trailing_zeros(),
            UnaryIntOp::PopCount => source.count_ones(),
        };
        ctx.write_register(return_value, result as u64);
        Ok(())
    }
}

impl InterpretInstr for TruncateIntInstr {
    fn interpret_instr(
        &self,
        _return_value: Option<Value>,
        _ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        unimplemented!()
    }
}

impl InterpretInstr for ExtendIntInstr {
    fn interpret_instr(
        &self,
        _return_value: Option<Value>,
        _ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        unimplemented!()
    }
}

impl InterpretInstr for IntToFloatInstr {
    fn interpret_instr(
        &self,
        _return_value: Option<Value>,
        _ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        unimplemented!()
    }
}

impl InterpretInstr for CompareIntInstr {
    fn interpret_instr(
        &self,
        return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        let return_value = return_value.expect("missing value for instruction");
        let lhs = ctx.read_register(self.lhs());
        let rhs = ctx.read_register(self.rhs());
        use CompareIntOp as Op;
        /// Compares `lhs` and `rhs` given the comparator `op` using `f` to convert to signed.
        fn cmp<U, S, F>(op: CompareIntOp, lhs: U, rhs: U, mut f: F) -> u64
        where
            U: Eq + Ord,
            S: Ord,
            F: FnMut(U) -> S,
        {
            let result = match op {
                Op::Eq => lhs == rhs,
                Op::Ne => lhs != rhs,
                Op::Slt => f(lhs) < f(rhs),
                Op::Sle => f(lhs) <= f(rhs),
                Op::Sgt => f(lhs) > f(rhs),
                Op::Sge => f(lhs) >= f(rhs),
                Op::Ult => lhs < rhs,
                Op::Ule => lhs <= rhs,
                Op::Ugt => lhs > rhs,
                Op::Uge => lhs >= rhs,
            };
            result as u64
        }
        let result = match self.ty() {
            IntType::I8 => {
                let lhs = lhs as u8;
                let rhs = rhs as u8;
                cmp(self.op(), lhs, rhs, |lhs| lhs as i8)
            }
            IntType::I16 => {
                let lhs = lhs as u16;
                let rhs = rhs as u16;
                cmp(self.op(), lhs, rhs, |lhs| lhs as i16)
            }
            IntType::I32 => {
                let lhs = lhs as u32;
                let rhs = rhs as u32;
                cmp(self.op(), lhs, rhs, |lhs| lhs as i32)
            }
            IntType::I64 => cmp(self.op(), lhs, rhs, |lhs| lhs as i64),
        };
        ctx.write_register(return_value, result);
        Ok(())
    }
}

/// Trait used to streamline operations on primitive types.
pub trait PrimitiveInteger: Copy {
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
    fn wrapping_mul(self, rhs: Self) -> Self;
    fn wrapping_div(self, rhs: Self) -> Self;
    fn wrapping_rem(self, rhs: Self) -> Self;
}
macro_rules! impl_primitive_integer_for {
    ( $( $type:ty ),* $(,)? ) => {
        $(
            impl PrimitiveInteger for $type {
                fn wrapping_add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }
                fn wrapping_sub(self, rhs: Self) -> Self { self.wrapping_sub(rhs) }
                fn wrapping_mul(self, rhs: Self) -> Self { self.wrapping_mul(rhs) }
                fn wrapping_div(self, rhs: Self) -> Self { self.wrapping_div(rhs) }
                fn wrapping_rem(self, rhs: Self) -> Self { self.wrapping_rem(rhs) }
            }
        )*
    };
}
impl_primitive_integer_for! {
    i8, i16, i32, i64,
    u8, u16, u32, u64,
}

impl InterpretInstr for BinaryIntInstr {
    fn interpret_instr(
        &self,
        return_value: Option<Value>,
        ctx: &mut InterpretationContext,
    ) -> Result<(), InterpretationError> {
        let return_value = return_value.expect("missing value for instruction");
        let lhs = ctx.read_register(self.lhs());
        let rhs = ctx.read_register(self.rhs());
        use core::ops::{BitAnd, BitOr, BitXor};
        use BinaryIntOp as Op;
        /// Computes `op` on `lhs` and `rhs` using `f` to convert from unsigned to signed.
        fn compute<U, S, F, V>(
            op: BinaryIntOp,
            lhs: U,
            rhs: U,
            mut u2s: F,
            mut s2u: V,
        ) -> U
        where
            U: PrimitiveInteger
                + BitAnd<Output = U>
                + BitOr<Output = U>
                + BitXor<Output = U>,
            S: PrimitiveInteger
                + BitAnd<Output = S>
                + BitOr<Output = S>
                + BitXor<Output = S>,
            F: FnMut(U) -> S,
            V: FnMut(S) -> U,
        {
            match op {
                Op::Add => lhs.wrapping_add(rhs),
                Op::Sub => lhs.wrapping_sub(rhs),
                Op::Mul => lhs.wrapping_mul(rhs),
                Op::Sdiv => s2u(u2s(lhs).wrapping_div(u2s(rhs))),
                Op::Srem => s2u(u2s(lhs).wrapping_rem(u2s(rhs))),
                Op::Udiv => lhs.wrapping_div(rhs),
                Op::Urem => lhs.wrapping_rem(rhs),
                Op::And => lhs & rhs,
                Op::Or => lhs | rhs,
                Op::Xor => lhs ^ rhs,
                _ => unimplemented!(),
            }
        }
        let result = match self.ty() {
            IntType::I8 => {
                let lhs = lhs as u8;
                let rhs = rhs as u8;
                let result =
                    compute(self.op(), lhs, rhs, |u| u as i8, |s| s as u8);
                result as u64
            }
            IntType::I16 => {
                let lhs = lhs as u16;
                let rhs = rhs as u16;
                let result =
                    compute(self.op(), lhs, rhs, |u| u as i16, |s| s as u16);
                result as u64
            }
            IntType::I32 => {
                let lhs = lhs as u32;
                let rhs = rhs as u32;
                let result =
                    compute(self.op(), lhs, rhs, |u| u as i32, |s| s as u32);
                result as u64
            }
            IntType::I64 => {
                let result =
                    compute(self.op(), lhs, rhs, |u| u as i64, |s| s as u64);
                result as u64
            }
        };
        ctx.write_register(return_value, result);
        Ok(())
    }
}