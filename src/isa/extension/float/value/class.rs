// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::isa::extension::float) enum Class {
    Finite,
    Infinite,
    Quiet,
    Signaling,
    Zero,
}
