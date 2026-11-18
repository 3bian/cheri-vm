// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExitLine {
    Hidden,
    #[default]
    Shown,
}
