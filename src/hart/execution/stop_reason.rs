// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StopReason {
    Breakpoint { address: u64 },
    BudgetExhausted,
    GuestCompletion { code: u64 },
    GuestEbreak { address: u64 },
}
