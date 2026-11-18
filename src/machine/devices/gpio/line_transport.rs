// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::error::HostResult;

pub(crate) trait LineTransport {
    fn read(&mut self) -> HostResult<u32>;
    fn write(&mut self, direction: u32, outputs: u32) -> HostResult<()>;
}
