// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::debug::gdb::{Core, Target};

impl<C: Core> Target<C> {
    pub(crate) fn core(&self) -> &C {
        &self.core
    }
}
