// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Presentation;

#[cfg(all(feature = "gdb", unix))]
impl<A: Presentation + crate::debug::gdb::GdbArchitecture> Runnable for A {}

#[cfg(not(all(feature = "gdb", unix)))]
impl<A: Presentation> Runnable for A {}

#[cfg(all(feature = "gdb", unix))]
pub(crate) trait Runnable:
    Presentation + crate::debug::gdb::GdbArchitecture
{
}

#[cfg(not(all(feature = "gdb", unix)))]
pub(crate) trait Runnable: Presentation {}
