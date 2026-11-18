// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

registers! {
    Register {
        Pending = 0, "pending";
        Enabled = 4, "enabled";
        ClaimComplete = 8, "claimComplete";
        InService = 12, "inService";
    }
}

pub(crate) const NO_SOURCE: u32 = 0;
pub(crate) const SOURCES: u32 = 32;
