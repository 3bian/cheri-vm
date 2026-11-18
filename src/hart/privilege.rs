// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod machine_only;
mod mode;
mod privilege_model;
mod reach;

pub use machine_only::MachineOnly;
pub use mode::Mode;
pub use privilege_model::PrivilegeModel;
pub use reach::Reach;

pub(crate) mod pmp;

mod supervisor;

pub use supervisor::Supervisor;

const INTERRUPT_PRIORITY: [u32; 5] = [11, 3, 7, 9, 1];

pub(crate) fn highest_interrupt(enabled: u32) -> u32 {
    INTERRUPT_PRIORITY
        .into_iter()
        .find(|cause| enabled & (1 << cause) != 0)
        .unwrap_or(5)
}
