// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::output_capture::*;

impl OutputCapture {
    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn take(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }
}
