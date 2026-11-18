// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

registers! {
    Register {
        Base = 0x00, "base";
        Top = 0x04, "top";
        Start = 0x08, "start";
        Epoch = 0x0c, "epoch";
        Status = 0x10, "status";
        InterruptRequest = 0x14, "interruptRequest";
    }
}

pub(crate) const IDENTITY: u32 = 0x5500_0000;
pub(crate) const INTERRUPT_REQUEST_ENABLE: u32 = 1 << 0;
