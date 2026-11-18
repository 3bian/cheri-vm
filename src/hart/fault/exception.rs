// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Fault;
use crate::machine::EmulatorError;

#[derive(Debug)]
pub enum Exception<W = u32> {
    Fatal(EmulatorError),
    Fault(Fault<W>),
}

impl<W> From<Fault<W>> for Exception<W> {
    fn from(fault: Fault<W>) -> Self {
        Self::Fault(fault)
    }
}
