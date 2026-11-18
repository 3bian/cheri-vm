// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::device_configuration::DeviceConfiguration;
use super::memory_configuration::MemoryConfiguration;
use crate::machine::completion::CompletionPolicy;
use crate::machine::timing_policy::TimingPolicy;

#[derive(Debug)]
pub struct MachineDescription {
    pub completion: Option<CompletionPolicy>,
    pub devices: Vec<DeviceConfiguration>,
    pub extensions: Option<Vec<String>>,
    pub memory: Vec<MemoryConfiguration>,
    pub name: String,
    pub processor: String,
    pub steps_per_tick: u32,
    pub timer_hz: u32,
    pub timing: TimingPolicy,
}
