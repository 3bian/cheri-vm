// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub struct Positional {
    pub help: &'static str,
    pub name: &'static str,
    pub unless: Option<&'static str>,
}
