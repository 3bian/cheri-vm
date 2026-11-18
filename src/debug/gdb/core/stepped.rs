// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Trap;

pub(crate) struct Stepped<F> {
    pub completion: Option<u64>,
    pub trap: Option<Trap<F>>,
}
