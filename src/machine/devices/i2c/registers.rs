// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

registers! {
    Register {
        Address = 0, "address";
        Data = 4, "data";
        Command = 8, "command";
        Status = 12, "status";
        Control = 16, "control";
        Acknowledge = 20, "acknowledge";
    }
}

pub(crate) const ADDRESS_MAX: u32 = 0x7f;
pub(crate) const COMMAND_READ: u32 = 2;
pub(crate) const COMMAND_RESTART: u32 = 4;
pub(crate) const COMMAND_STOP: u32 = 3;
pub(crate) const COMMAND_WRITE: u32 = 1;
pub(crate) const CONTROL_INTERRUPT_ENABLE: u32 = 1 << 0;
pub(crate) const CONTROL_READ_LENGTH: u32 = 0x0fff << CONTROL_READ_LENGTH_SHIFT;
pub(crate) const CONTROL_READ_LENGTH_SHIFT: u32 = 16;
pub(crate) const STATUS_ACKNOWLEDGED: u32 = 1 << 0;
pub(crate) const STATUS_ACTIVE: u32 = 1 << 1;
pub(crate) const STATUS_COMPLETE: u32 = 1 << 2;
pub(crate) const STATUS_ERROR: u32 = 1 << 3;
