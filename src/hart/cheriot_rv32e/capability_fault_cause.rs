// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::Fault;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub(crate) enum CapabilityFaultCause {
    Bounds = 1,
    Tag = 2,
    Seal = 3,
    ExecutePermission = 17,
    LoadPermission = 18,
    StorePermission = 19,
    StoreCapabilityPermission = 21,
    SystemRegisterPermission = 24,
}

impl Fault {
    pub(crate) fn capability(code: u32, register: u8) -> Self {
        Self {
            cause: 28,
            value: (u32::from(register) << 5) | code,
        }
    }

    pub(crate) fn capability_cause(
        cause: CapabilityFaultCause,
        register: u8,
    ) -> Self {
        Self::capability(cause as u32, register)
    }
}
