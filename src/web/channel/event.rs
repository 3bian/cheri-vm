// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Event {
    Binary,
    Close { code: Option<u16>, reason: String },
    Text(String),
}
