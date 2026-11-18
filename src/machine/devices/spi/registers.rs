// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

registers! {
    Register {
        Data = 0, "data";
        Status = 4, "status";
        ChipSelect = 8, "chipSelect";
        Mode = 12, "mode";
        Control = 16, "control";
        Acknowledge = 20, "acknowledge";
    }
}

pub(crate) const CHIP_SELECT_COUNT: u32 = 4;
pub(crate) const CONTROL_INTERRUPT_ENABLE: u32 = 1 << 0;
pub(crate) const DESELECTED: u32 = u32::MAX;
pub(crate) const MODE_MASK: u32 = 3;
pub(crate) const STATUS_COMPLETE: u32 = 1 << 0;
pub(crate) const STATUS_MISSING_PERIPHERAL: u32 = 1 << 1;
