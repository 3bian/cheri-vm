// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug)]
pub enum Presentation {
    Escaped,
    Labeled,
    Raw,
}
