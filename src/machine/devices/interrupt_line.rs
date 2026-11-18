// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum InterruptLine {
    MachineExternal,
    MachineSoftware,
    MachineTimer,
    SupervisorExternal,
}
