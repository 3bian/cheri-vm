// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Metadata;
use crate::isa::pointer::rvy::{Capability128, Window, permissions};

pub struct Shadow {
    pub exception_program_counters: [Metadata; 2],
    pub levels: bool,
    pub(super) program_counter: Metadata,
    pub registers: [Metadata; 32],
    pub scratches: [Metadata; 2],
    pub thread_identifiers: [Capability128; 3],
    pub trap_vectors: [Metadata; 2],
    pub(super) window: Window,
}

impl Shadow {
    pub(super) fn enter(&mut self, capability: Capability128) {
        self.program_counter = Metadata::of(capability);
        self.window = Window::of(capability);
    }

    pub fn program_counter(&self) -> Metadata {
        self.program_counter
    }
}

impl Shadow {
    pub fn seen_permissions(&self, capability: Capability128) -> u64 {
        if self.levels {
            capability.permissions()
        } else {
            capability.permissions() | permissions::LEVELS
        }
    }
}
