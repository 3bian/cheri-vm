// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod aes_byte_operation;
mod atomic_operation;
mod branch_condition;
mod csr_operation;
mod float_operation;
mod immediate_operation;
mod load_kind;
mod register_operation;
mod store_kind;

pub use aes_byte_operation::AesByteOperation;
pub use atomic_operation::{AtomicOperation, AtomicWidth};
pub use branch_condition::BranchCondition;
pub use csr_operation::CsrOperation;
pub use float_operation::{FloatOperation, FloatWidth};
pub use immediate_operation::ImmediateOperation;
pub use load_kind::LoadKind;
pub use register_operation::RegisterOperation;
pub use store_kind::StoreKind;

use super::extension::cheriot::{Operation, RegisterPair};
use super::extension::rvy;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Instruction {
    AddUpperImmediateToProgramCounter {
        destination: u8,
        value: u32,
    },
    AesByte {
        operation: AesByteOperation,
        destination: u8,
        first: u8,
        second: u8,
        select: u8,
    },
    Atomic {
        operation: AtomicOperation,
        width: AtomicWidth,
        destination: u8,
        address: u8,
        source: u8,
    },
    Branch {
        condition: BranchCondition,
        first: u8,
        second: u8,
        offset: i32,
    },
    Breakpoint,
    Capability {
        operation: Operation,
        registers: RegisterPair,
        second: u8,
        immediate: i32,
    },
    Csr {
        operation: CsrOperation,
        destination: u8,
        source: u8,
        address: u16,
        immediate: bool,
    },
    EnvironmentCall,
    Fence,
    FenceInstruction,
    Float {
        operation: FloatOperation,
        width: FloatWidth,
        destination: u8,
        first: u8,
        second: u8,
        third: u8,
        rounding: u8,
    },
    FloatLoad {
        width: FloatWidth,
        destination: u8,
        base: u8,
        offset: i32,
    },
    FloatStore {
        width: FloatWidth,
        source: u8,
        base: u8,
        offset: i32,
    },
    Hint,
    Immediate {
        operation: ImmediateOperation,
        destination: u8,
        source: u8,
        immediate: i32,
    },
    ImmediateWord {
        operation: ImmediateOperation,
        destination: u8,
        source: u8,
        immediate: i32,
    },
    JumpAndLink {
        destination: u8,
        offset: i32,
    },
    JumpAndLinkRegister {
        destination: u8,
        base: u8,
        offset: i32,
    },
    Load {
        kind: LoadKind,
        destination: u8,
        base: u8,
        offset: i32,
    },
    LoadUpperImmediate {
        destination: u8,
        value: u32,
    },
    MachineReturn,
    Register {
        operation: RegisterOperation,
        destination: u8,
        first: u8,
        second: u8,
    },
    RegisterWord {
        operation: RegisterOperation,
        destination: u8,
        first: u8,
        second: u8,
    },
    Rvy {
        operation: rvy::Operation,
        destination: u8,
        first: u8,
        second: u8,
        immediate: i16,
    },
    Store {
        kind: StoreKind,
        source: u8,
        base: u8,
        offset: i32,
    },
    SupervisorFence,
    SupervisorReturn,
    WaitForInterrupt,
}

impl Instruction {
    pub(crate) fn csr_write_address(self) -> Option<u16> {
        match self {
            Self::Csr {
                address,
                operation,
                source,
                ..
            } if operation.writes(source) => Some(address),
            _ => None,
        }
    }
}
