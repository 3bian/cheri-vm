// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Trapped;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Advanced {
    pub exit: Option<u64>,
    pub trap: Option<Trapped>,
}
