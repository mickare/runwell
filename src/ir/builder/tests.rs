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

use super::Function;
use crate::{
    entity::RawIdx,
    ir::{
        instruction::CompareIntOp,
        primitive::{IntConst, IntType},
        IrError,
        Variable,
    },
};

#[test]
fn ret_const_works() -> Result<(), IrError> {
    let mut b = Function::build()
        .with_inputs(&[])?
        .with_outputs(&[])?
        .body();
    let c = b.ins()?.constant(IntConst::I32(42))?;
    b.ins()?.return_value(c)?;
    let fun = b.finalize()?;
    println!("{}", fun);
    Ok(())
}

#[test]
fn simple_block_works() -> Result<(), IrError> {
    let mut b = Function::build()
        .with_inputs(&[])?
        .with_outputs(&[])?
        .body();
    let v1 = b.ins()?.constant(IntConst::I32(1))?;
    let v2 = b.ins()?.constant(IntConst::I32(2))?;
    let v3 = b.ins()?.iadd(IntType::I32, v1, v2)?;
    let v3 = b.ins()?.imul(IntType::I32, v3, v3)?;
    b.ins()?.return_value(v3)?;
    let fun = b.finalize()?;
    println!("{}", fun);
    Ok(())
}

#[test]
fn if_then_else_works() -> Result<(), IrError> {
    let mut b = Function::build()
        .with_inputs(&[])?
        .with_outputs(&[])?
        .body();
    let then_block = b.create_block();
    let else_block = b.create_block();
    let v1 = b.ins()?.constant(IntConst::I32(1))?;
    let v2 = b.ins()?.constant(IntConst::I32(2))?;
    let v3 = b.ins()?.icmp(IntType::I32, CompareIntOp::Ule, v1, v2)?;
    let _v4 = b.ins()?.if_then_else(v3, then_block, else_block)?;
    b.switch_to_block(then_block)?;
    let v5 = b.ins()?.constant(IntConst::I32(10))?;
    b.ins()?.return_value(v5)?;
    b.seal_block()?;
    b.switch_to_block(else_block)?;
    let v6 = b.ins()?.constant(IntConst::I32(20))?;
    b.ins()?.return_value(v6)?;
    b.seal_block()?;
    let fun = b.finalize()?;
    println!("{}", fun);
    Ok(())
}

#[test]
fn simple_variable() -> Result<(), IrError> {
    let mut b = Function::build()
        .with_inputs(&[])?
        .with_outputs(&[])?
        .declare_variables(1, IntType::I32.into())?
        .body();
    let var = Variable::from_raw(RawIdx::from_u32(0));
    let v1 = b.ins()?.constant(IntConst::I32(1))?;
    b.write_var(var, v1)?;
    let v2 = b.read_var(var)?;
    let v3 = b.ins()?.iadd(IntType::I32, v2, v2)?;
    b.ins()?.return_value(v3)?;
    let fun = b.finalize()?;
    println!("{}", fun);
    Ok(())
}

#[test]
fn simple_input() -> Result<(), IrError> {
    let mut b = Function::build()
        .with_inputs(&[IntType::I32.into()])?
        .with_outputs(&[IntType::I32.into()])?
        .body();
    let input = Variable::from_raw(RawIdx::from_u32(0));
    let v0 = b.read_var(input)?;
    let v1 = b.ins()?.iadd(IntType::I32, v0, v0)?;
    b.ins()?.return_value(v1)?;
    let fun = b.finalize()?;
    println!("{}", fun);
    Ok(())
}

#[test]
fn simple_gvn_var_read() -> Result<(), IrError> {
    let mut b = Function::build()
        .with_inputs(&[IntType::I32.into()])?
        .with_outputs(&[])?
        .body();
    let var = Variable::from_raw(RawIdx::from_u32(0));
    let v0 = b.ins()?.constant(IntConst::I32(1))?;
    b.write_var(var, v0)?;
    let exit_block = b.create_block();
    b.ins()?.br(exit_block)?;
    b.switch_to_block(exit_block)?;
    let v0 = b.read_var(var)?;
    b.ins()?.return_value(v0)?;
    b.seal_block()?;
    let fun = b.finalize()?;
    println!("{}", fun);
    Ok(())
}

#[test]
fn simple_gvn_if_works() -> Result<(), IrError> {
    let mut b = Function::build()
        .with_inputs(&[IntType::I32.into()])?
        .with_outputs(&[])?
        .declare_variables(1, IntType::I32.into())?
        .body();

    let then_block = b.create_block();
    let else_block = b.create_block();
    let exit_block = b.create_block();

    let input = Variable::from_raw(RawIdx::from_u32(0));
    let var = Variable::from_raw(RawIdx::from_u32(1));

    let v0 = b.read_var(input)?;
    let v1 = b.ins()?.constant(IntConst::I32(0))?;
    let v2 = b.ins()?.icmp(IntType::I32, CompareIntOp::Eq, v0, v1)?;
    b.ins()?.if_then_else(v2, then_block, else_block)?;

    b.switch_to_block(then_block)?;
    let v3 = b.ins()?.constant(IntConst::I32(10))?;
    b.write_var(var, v3)?;
    b.ins()?.br(exit_block)?;
    b.seal_block()?;

    b.switch_to_block(else_block)?;
    let v4 = b.ins()?.constant(IntConst::I32(20))?;
    b.write_var(var, v4)?;
    b.ins()?.br(exit_block)?;
    b.seal_block()?;

    b.switch_to_block(exit_block)?;
    let v5 = b.read_var(var)?;
    b.ins()?.return_value(v5)?;
    b.seal_block()?;

    let fun = b.finalize()?;
    println!("{}", fun);

    Ok(())
}

#[test]
fn simple_loop_works() -> Result<(), IrError> {
    let mut b = Function::build()
        .with_inputs(&[IntType::I32.into()])?
        .with_outputs(&[])?
        .declare_variables(1, IntType::I32.into())?
        .body();

    let loop_head = b.create_block();
    let loop_body = b.create_block();
    let loop_exit = b.create_block();

    let input = Variable::from_raw(RawIdx::from_u32(0));
    let counter = Variable::from_raw(RawIdx::from_u32(1));

    let v0 = b.ins()?.constant(IntConst::I32(0))?;
    b.write_var(counter, v0)?;
    b.ins()?.br(loop_head)?;

    b.switch_to_block(loop_head)?;
    let v1 = b.read_var(counter)?;
    let v2 = b.read_var(input)?;
    let v3 = b.ins()?.icmp(IntType::I32, CompareIntOp::Slt, v1, v2)?;
    b.ins()?.if_then_else(v3, loop_body, loop_exit)?;

    b.switch_to_block(loop_body)?;
    let v4 = b.read_var(counter)?;
    let v5 = b.ins()?.constant(IntConst::I32(1))?;
    let v6 = b.ins()?.iadd(IntType::I32, v4, v5)?;
    b.write_var(counter, v6)?;
    b.ins()?.br(loop_head)?;
    b.seal_block()?;

    b.switch_to_block(loop_head)?;
    b.seal_block()?;

    b.switch_to_block(loop_exit)?;
    let v7 = b.read_var(counter)?;
    b.ins()?.return_value(v7)?;
    b.seal_block()?;

    let fun = b.finalize()?;
    println!("{}", fun);

    Ok(())
}
