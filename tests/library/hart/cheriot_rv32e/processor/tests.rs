// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#![cfg(feature = "verification")]

use crate::hart::cheriot_rv32e::Processor;
use crate::hart::cheriot_rv32e::RegisterAccess;

impl Processor {
    pub(crate) fn register_accesses(&self) -> Vec<RegisterAccess> {
        self.register_accesses
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .1
            .clone()
    }
}
