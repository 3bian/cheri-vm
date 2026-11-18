// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Default)]
pub(crate) struct SupervisorRegisters {
    pub(crate) cause: u64,
    pub(crate) counter_enable: u32,
    pub(crate) exception_program_counter: u64,
    pub(crate) scratch: u64,
    pub(crate) trap_value: u64,
    pub(crate) trap_vector: u64,
}
