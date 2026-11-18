// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::Hart;
use crate::hart::plain::Plain;
use crate::hart::plain::model::Model;
use crate::hart::privilege::{PrivilegeModel, Reach};
use crate::isa::extension::float::arithmetic::{
    add, divide, fused_multiply_add, multiply, square_root, subtract,
};
use crate::isa::extension::float::compare::{
    classify, equal, less_or_equal, less_than, maximum, minimum,
};
use crate::isa::extension::float::convert::{
    from_integer, resized, to_integer,
};
use crate::isa::extension::float::{Format, RoundingMode};
use crate::isa::instruction::{FloatOperation, FloatWidth, Instruction};
use crate::isa::pointer::Access;
use crate::isa::xlen::{Word, Xlen};
use crate::machine::error::EmulatorError;

const BOX: u64 = 0xffff_ffff << 32;

impl<X: Xlen, P: PrivilegeModel, M: Model<X>> Hart<Plain<X, P, M>> {
    fn computed(
        &self,
        operation: FloatOperation,
        width: FloatWidth,
        [first, second, third]: [u8; 3],
        mode: RoundingMode,
        flags: &mut u8,
    ) -> Outcome {
        use FloatOperation::*;
        let format = width.format();
        let raw =
            |index: u8| self.processor.float_registers[usize::from(index)];
        let value = |index: u8| held(raw(index), width);
        let whole = self.processor.read_register_unchecked(first).to_u64();
        let (a, b) = (value(first), value(second));
        let sign = format.sign();
        let float = match operation {
            Add => add(format, a, b, mode, flags),
            Subtract => subtract(format, a, b, mode, flags),
            Multiply => multiply(format, a, b, mode, flags),
            Divide => divide(format, a, b, mode, flags),
            SquareRoot => square_root(format, a, mode, flags),
            SignInject => (a & !sign) | (b & sign),
            SignInjectNegated => (a & !sign) | (!b & sign),
            SignInjectXor => a ^ (b & sign),
            Minimum => minimum(format, a, b, flags),
            Maximum => maximum(format, a, b, flags),
            MultiplyAdd
            | MultiplySubtract
            | NegatedMultiplySubtract
            | NegatedMultiplyAdd => {
                let negations = [
                    matches!(
                        operation,
                        NegatedMultiplySubtract | NegatedMultiplyAdd
                    ),
                    matches!(operation, MultiplySubtract | NegatedMultiplyAdd),
                ];
                fused_multiply_add(
                    format,
                    [a, b, value(third)],
                    negations,
                    mode,
                    flags,
                )
            }
            FromWord => from_integer(format, whole, true, 32, mode, flags),
            FromUnsignedWord => {
                from_integer(format, whole, false, 32, mode, flags)
            }
            FromLong => from_integer(format, whole, true, 64, mode, flags),
            FromUnsignedLong => {
                from_integer(format, whole, false, 64, mode, flags)
            }
            Resize => match width {
                FloatWidth::Single => {
                    resized(Format::DOUBLE, format, raw(first), mode, flags)
                }
                FloatWidth::Double => resized(
                    Format::SINGLE,
                    format,
                    held(raw(first), FloatWidth::Single),
                    mode,
                    flags,
                ),
            },
            MoveFromInteger => match width {
                FloatWidth::Single => whole & !BOX,
                FloatWidth::Double => whole,
            },
            integer => {
                return Outcome::Integer(match integer {
                    Equal => u64::from(equal(format, a, b, flags)),
                    LessThan => u64::from(less_than(format, a, b, flags)),
                    LessOrEqual => {
                        u64::from(less_or_equal(format, a, b, flags))
                    }
                    Classify => classify(format, a),
                    ToWord => to_integer(format, a, true, 32, mode, flags),
                    ToUnsignedWord => {
                        to_integer(format, a, false, 32, mode, flags)
                    }
                    ToLong => to_integer(format, a, true, 64, mode, flags),
                    ToUnsignedLong => {
                        to_integer(format, a, false, 64, mode, flags)
                    }
                    _ => match width {
                        FloatWidth::Single => {
                            i64::from(raw(first) as i32) as u64
                        }
                        FloatWidth::Double => raw(first),
                    },
                });
            }
        };
        Outcome::Float(float)
    }

    pub(super) fn float(
        &mut self,
        instruction: Instruction,
        address: X::Word,
        following: X::Word,
    ) -> Result<X::Word, EmulatorError> {
        if !self.processor.float_enabled() {
            return Err(refused());
        }
        match instruction {
            Instruction::FloatLoad {
                width,
                destination,
                base,
                offset,
            } => {
                let target = self
                    .processor
                    .read_register_unchecked(base)
                    .add_signed(offset);
                self.check_alignment(
                    address,
                    base,
                    target,
                    width.bytes(),
                    Access::Load,
                )?;
                let value = self.load(target, width.bytes(), Reach::Load)?;
                self.processor.float_registers[usize::from(destination)] =
                    kept(value, width);
                self.processor.touch_float();
            }
            Instruction::FloatStore {
                width,
                source,
                base,
                offset,
            } => {
                let target = self
                    .processor
                    .read_register_unchecked(base)
                    .add_signed(offset);
                self.check_alignment(
                    address,
                    base,
                    target,
                    width.bytes(),
                    Access::Store,
                )?;
                let value = self.processor.float_registers[usize::from(source)];
                self.store(target, width.bytes(), value)?;
            }
            Instruction::Float {
                operation,
                width,
                destination,
                first,
                second,
                third,
                rounding,
            } => {
                let chosen = if rounding == 7 {
                    (self.processor.float_control >> 5) & 7
                } else {
                    u32::from(rounding)
                };
                let mode = match RoundingMode::from_bits(chosen) {
                    Some(mode) => mode,
                    None if operation.rounds() => return Err(refused()),
                    None => RoundingMode::NearestEven,
                };
                let mut flags = 0;
                let outcome = self.computed(
                    operation,
                    width,
                    [first, second, third],
                    mode,
                    &mut flags,
                );
                match outcome {
                    Outcome::Float(value) => {
                        self.processor.float_registers
                            [usize::from(destination)] = kept(value, width)
                    }
                    Outcome::Integer(value) => {
                        self.processor.write_register_unchecked(
                            destination,
                            X::Word::from_u64(value),
                        )
                    }
                }
                self.processor.float_control |= u32::from(flags);
                self.processor.touch_float();
            }
            _ => {
                return Err(EmulatorError::Execution(
                    "not a floating-point instruction".to_owned(),
                ));
            }
        }
        Ok(following)
    }
}

enum Outcome {
    Float(u64),
    Integer(u64),
}

fn held(register: u64, width: FloatWidth) -> u64 {
    match width {
        FloatWidth::Double => register,
        FloatWidth::Single if register & BOX == BOX => register & !BOX,
        FloatWidth::Single => Format::SINGLE.canonical_nan(),
    }
}

fn kept(value: u64, width: FloatWidth) -> u64 {
    match width {
        FloatWidth::Double => value,
        FloatWidth::Single => value | BOX,
    }
}

fn refused() -> EmulatorError {
    EmulatorError::Refused(
        "floating point is off, or its rounding mode is not one".into(),
    )
}
