// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

registers! {
    Register {
        SoftwareInterrupt = 0x0000, "msip";
        CompareLow = 0x4000, "mtimecmp";
        CompareHigh = 0x4004, "mtimecmph";
        TimeLow = 0xbff8, "mtime";
        TimeHigh = 0xbffc, "mtimeh";
    }
}

pub(crate) const SOFTWARE_INTERRUPT_PENDING: u32 = 1 << 0;
