// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Described;
use crate::hart::{Architecture, Hart};
use crate::machine::bus::Bus;

pub(crate) trait DebugArchitecture: Architecture {
    const THREAD: &'static str;

    fn described_registers(processor: &Self::Processor) -> Vec<Described>;

    fn monitor(
        _hart: &Hart<Self>,
        _words: &[&str],
    ) -> Option<Result<String, &'static str>> {
        None
    }

    fn physical(
        _processor: &Self::Processor,
        _bus: &Bus,
        address: u64,
    ) -> Option<u64> {
        Some(address)
    }

    fn return_address(processor: &Self::Processor) -> Option<u64>;
}
