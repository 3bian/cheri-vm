// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Default)]
pub(crate) struct OutputCapture {
    pub(crate) bytes: Vec<u8>,
}

impl OutputCapture {
    const CAPACITY: usize = 65536;

    pub(crate) fn drain_into(&mut self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.bytes);
        self.bytes.clear();
    }

    pub(crate) fn has_capacity(&self) -> bool {
        self.bytes.len() < Self::CAPACITY
    }

    pub(crate) fn push(&mut self, byte: u8) -> bool {
        if !self.has_capacity() {
            return false;
        }
        self.bytes.push(byte);
        true
    }
}
