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

use crate::{
    primitive::{IntType, Value},
    VisitValues,
    VisitValuesMut,
};
use core::fmt::Display;

/// The base of all binary integer instructions.
///
/// Generic over a concrete binary integer operand.
///
/// # Note
///
/// - Both input values and the output value of the instruction are
///   equal to the type `ty`.
/// - In case of shift and rotate operands the `lhs` value is the source
///   and the `rhs` value is the shift or rotate amount.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BinaryIntInstr {
    op: BinaryIntOp,
    ty: IntType,
    lhs: Value,
    rhs: Value,
}

/// Binary integer operand codes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinaryIntOp {
    /// Evaluates integer addition of two integer values.
    Add,
    /// Subtracts the right-hand side integer from the left-hand side integer.
    Sub,
    /// Evaluates integer multiplication of two integer values.
    Mul,
    /// Divides the right-hand side signed integer from the left-hand side signed integer.
    Sdiv,
    /// Divides the right-hand side unsigned integer from the left-hand side unsigned integer.
    Udiv,
    /// Computes the remainder of the left-hand side signed integer divided by the right-hand side signed integer.
    Srem,
    /// Computes the remainder of the left-hand side unsigned integer divided by the right-hand side unsigned integer.
    Urem,
    /// Computes the bit-wise and for two integer value.
    And,
    /// Computes the bit-wise or for two integer value.
    Or,
    /// Computes the bit-wise xor for two integer value.
    Xor,
}

impl Display for BinaryIntOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let repr = match self {
            Self::Add => "iadd",
            Self::Sub => "isub",
            Self::Mul => "imul",
            Self::Sdiv => "sdiv",
            Self::Udiv => "udiv",
            Self::Srem => "srem",
            Self::Urem => "urem",
            Self::And => "iand",
            Self::Or => "ior",
            Self::Xor => "ixor",
        };
        write!(f, "{}", repr)?;
        Ok(())
    }
}

impl BinaryIntInstr {
    /// Creates a new binary integer instruction.
    pub fn new(op: BinaryIntOp, ty: IntType, lhs: Value, rhs: Value) -> Self {
        Self { op, ty, lhs, rhs }
    }

    /// Returns the binary operand of the instruction.
    #[inline]
    pub fn op(&self) -> BinaryIntOp {
        self.op
    }

    /// Returns the left-hand side value.
    #[inline]
    pub fn lhs(&self) -> Value {
        self.lhs
    }

    /// Returns the right-hand side value.
    #[inline]
    pub fn rhs(&self) -> Value {
        self.rhs
    }

    /// Returns the integer type of the instruction.
    #[inline]
    pub fn ty(&self) -> IntType {
        self.ty
    }
}

impl VisitValues for BinaryIntInstr {
    fn visit_values<V>(&self, mut visitor: V)
    where
        V: FnMut(Value) -> bool,
    {
        let _ = visitor(self.lhs) && visitor(self.rhs);
    }
}

impl VisitValuesMut for BinaryIntInstr {
    fn visit_values_mut<V>(&mut self, mut visitor: V)
    where
        V: FnMut(&mut Value) -> bool,
    {
        let _ = visitor(&mut self.lhs) && visitor(&mut self.rhs);
    }
}

impl Display for BinaryIntInstr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}<{}> {} {}", self.op, self.ty, self.lhs, self.rhs)?;
        Ok(())
    }
}
