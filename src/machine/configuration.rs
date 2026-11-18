// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod machine_configuration;
mod machine_description;

pub use machine_configuration::MachineConfiguration;
pub(crate) use machine_description::MachineDescription;

mod default_devices;

mod device_configuration;

mod extensions;

mod memory_configuration;

pub(crate) use device_configuration::DeviceConfiguration;

pub(crate) use memory_configuration::MemoryConfiguration;
