// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::arithmetic::*;
use crate::isa::instruction::ImmediateOperation;
use crate::isa::instruction::RegisterOperation;

#[test]
fn immediates_are_sign_extended_before_use() {
    assert_eq!(
        perform_immediate_operation(ImmediateOperation::Add, 0_u32, -1),
        u32::MAX
    );
    assert_eq!(
        perform_immediate_operation(
            ImmediateOperation::SetLessThanUnsigned,
            1_u32,
            -1
        ),
        1
    );
}

#[test]
fn multiplication_and_division_handle_their_edge_cases() {
    assert_eq!(
        perform_register_operation(
            RegisterOperation::MulHighUnsigned,
            u32::MAX,
            u32::MAX
        ),
        0xffff_fffe
    );
    assert_eq!(
        perform_register_operation(
            RegisterOperation::MulHighSignedUnsigned,
            u32::MAX,
            u32::MAX
        ),
        u32::MAX
    );
    assert_eq!(
        perform_register_operation(
            RegisterOperation::DivSigned,
            i32::MIN as u32,
            u32::MAX
        ),
        i32::MIN as u32
    );
    assert_eq!(
        perform_register_operation(
            RegisterOperation::RemainderSigned,
            i32::MIN as u32,
            u32::MAX
        ),
        0
    );
    assert_eq!(
        perform_register_operation(RegisterOperation::DivUnsigned, 1_u32, 0),
        u32::MAX
    );
    assert_eq!(
        perform_register_operation(
            RegisterOperation::RemainderUnsigned,
            42_u32,
            0
        ),
        42
    );
}
