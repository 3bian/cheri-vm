// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UartLayout {
    Ns16550,
    #[default]
    Simple,
}
