// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

registers! {
    Register {
        Count = 0, "count";
        Compare = 4, "compare";
        Control = 8, "control";
        Pending = 12, "pending";
    }
}

pub(crate) const CONTROL_ENABLE: u32 = 1 << 0;
pub(crate) const CONTROL_INTERRUPT_ENABLE: u32 = 1 << 1;
pub(crate) const PENDING_ACKNOWLEDGE: u32 = 1 << 0;
