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

use crate::ir::{FloatType, Value};
use core::{fmt::Display, marker::PhantomData};

/// The base of all binary floating point number instructions.
///
/// Generic over a concrete binary floating point number operand.
#[derive(Debug, PartialEq, Eq)]
pub struct BinaryFloatInstr<T>
where
    T: BinaryFloatOperand,
{
    ty: FloatType,
    lhs: Value,
    rhs: Value,
    marker: PhantomData<fn() -> T>,
}

impl<T> Display for BinaryFloatInstr<T>
where
    T: BinaryFloatOperand,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} type {}, lhs {}, rhs {}",
            <T as BinaryFloatOperand>::DISPLAY_REPR,
            self.ty,
            self.lhs,
            self.rhs
        )?;
        Ok(())
    }
}

mod operands {
    /// Types implementing this trait are binary integer instruction operands.
    pub trait BinaryFloatOperand: Sealed {
        /// A string representation for `Display` trait implementations.
        const DISPLAY_REPR: &'static str;
        /// Is `true` if the operation is commutative, i.e. identical upon swapping `lhs` and `rhs`.
        const COMMUTATIVE: bool;
    }
    pub trait Sealed {}

    macro_rules! impl_binary_float_operand {
        (
            $( #[$attr:meta] )*
            struct $name:ident {
                commutative: $commutative:literal,
                display_repr: $display_repr:literal
            }
        ) => {
            $( #[$attr] )*
            #[derive(Debug, Copy, Clone, PartialEq, Eq)]
            pub enum $name {}

            impl BinaryFloatOperand for $name {
                const DISPLAY_REPR: &'static str = $display_repr;
                const COMMUTATIVE: bool = $commutative;
            }
            impl Sealed for $name {}
        };
    }

    impl_binary_float_operand! {
        /// Binary operand for floating point number addition.
        struct Add {
            commutative: true,
            display_repr: "fadd"
        }
    }
    impl_binary_float_operand! {
        /// Binary operand for floating point number subtraction.
        struct Sub {
            commutative: false,
            display_repr: "sub"
        }
    }
    impl_binary_float_operand! {
        /// Binary operand for floating point number multiplication.
        struct Mul {
            commutative: true,
            display_repr: "mul"
        }
    }
    impl_binary_float_operand! {
        /// Binary operand for floating point number division.
        struct Div {
            commutative: false,
            display_repr: "div"
        }
    }
    impl_binary_float_operand! {
        /// Binary operand for evaluating the minimum element of two floating point numbers.
        struct Min {
            commutative: true,
            display_repr: "min"
        }
    }
    impl_binary_float_operand! {
        /// Binary operand for evaluating the maximum element of two floating point numbers.
        struct Max {
            commutative: true,
            display_repr: "max"
        }
    }
    impl_binary_float_operand! {
        /// Binary operand for performing the copysign operation for two floating point numbers.
        ///
        /// # Note
        ///
        /// This is a bitwise instruction; it combines the sign bit from the second operand with all
        /// bits other than the sign bit from the first operand, even if either operand is a NaN or a zero.
        struct Copysign {
            commutative: false,
            display_repr: "copysign"
        }
    }
}
use self::operands::BinaryFloatOperand;

pub type FaddInstr = BinaryFloatInstr<operands::Add>;
pub type FsubInstr = BinaryFloatInstr<operands::Sub>;
pub type FmulInstr = BinaryFloatInstr<operands::Mul>;
pub type FdivInstr = BinaryFloatInstr<operands::Div>;
pub type FminInstr = BinaryFloatInstr<operands::Min>;
pub type FmaxInstr = BinaryFloatInstr<operands::Max>;
pub type FcopysignInstr = BinaryFloatInstr<operands::Copysign>;