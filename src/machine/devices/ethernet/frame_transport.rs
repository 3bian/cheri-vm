// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::error::HostResult;

pub(crate) trait FrameTransport {
    fn receive(&mut self, frame: &mut [u8]) -> HostResult<usize>;
    fn transmit(&mut self, frame: &[u8]) -> HostResult<usize>;
}
