// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Described {
    pub address: Option<u64>,
    pub name: String,
    pub value: String,
}
