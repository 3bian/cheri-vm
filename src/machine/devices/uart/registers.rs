// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

registers! {
    Register {
        Data = 0, "data";
        Status = 4, "status";
        Control = 8, "control";
    }
}

registers! {
    Ns16550Register {
        Data = 0x00, "data";
        InterruptEnable = 0x04, "interruptEnable";
        InterruptIdentity = 0x08, "interruptIdentity";
        LineControl = 0x0c, "lineControl";
        ModemControl = 0x10, "modemControl";
        LineStatus = 0x14, "lineStatus";
        ModemStatus = 0x18, "modemStatus";
        Scratch = 0x1c, "scratch";
    }
}

pub(crate) const CONTROL_INTERRUPT_ENABLE: u32 = 1 << 0;
pub(crate) const ENABLE_RECEIVED: u32 = 1 << 0;
pub(crate) const ENABLE_TRANSMITTED: u32 = 1 << 1;
pub(crate) const IDENTITY_NONE: u32 = 1;
pub(crate) const IDENTITY_RECEIVED: u32 = 4;
pub(crate) const IDENTITY_TRANSMITTED: u32 = 2;
pub(crate) const LINE_CONTROL_DIVISOR_LATCH: u32 = 1 << 7;
pub(crate) const LINE_STATUS_DATA_READY: u32 = 1 << 0;
pub(crate) const LINE_STATUS_TRANSMIT_EMPTY: u32 = 3 << 5;
pub(crate) const NS16550_BYTES: u32 = 0x100;
pub(crate) const STATUS_RECEIVE_READY: u32 = 1 << 0;
pub(crate) const STATUS_TRANSMIT_READY: u32 = 1 << 1;
