// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod completion;
mod emulator_settings;
mod memory_entry;
mod profile_device;
mod timing;

use completion::Completion;
pub(crate) use emulator_settings::EmulatorSettings;
pub(crate) use memory_entry::MemoryEntry;
pub(crate) use profile_device::ProfileDevice;
use timing::Timing;

use crate::machine::EmulatorError;

use crate::machine::completion::CompletionPolicy;

use crate::machine::configuration::{
    DeviceConfiguration, MachineConfiguration, MachineDescription,
    MemoryConfiguration,
};

use crate::machine::devices::DeviceKind;

use crate::machine::memory::Tagging;

use crate::machine::timing_policy::{
    DEFAULT_STEPS_PER_TICK, DEFAULT_TIMER_HZ, TimingPolicy,
};

const DEFAULT_SECTORS: usize = 16;
pub const MAX_PROFILE_BYTES: usize = 1024 * 1024;

fn default_memory_kind() -> String {
    "ram".into()
}

fn default_steps_per_tick() -> u32 {
    DEFAULT_STEPS_PER_TICK
}

fn description(text: &str) -> Result<MachineDescription, EmulatorError> {
    {
        let table: toml::Table = toml::from_str(text).map_err(invalid)?;
        let name = table
            .get("name")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| invalid("profile needs a name"))?
            .to_owned();
        let settings: EmulatorSettings = table
            .get("emulator")
            .ok_or_else(|| {
                invalid(format!(
                    "profile {name} has no [emulator] table, so the emulator \
                     cannot run it"
                ))
            })?
            .clone()
            .try_into()
            .map_err(|error| invalid(format!("[emulator]: {error}")))?;
        let interrupt_number = |interrupt: &str| -> Result<u8, EmulatorError> {
            table
                .get("interrupt")
                .and_then(|interrupts| interrupts.get(interrupt))
                .and_then(|entry| entry.get("number"))
                .and_then(toml::Value::as_integer)
                .and_then(|number| u8::try_from(number).ok())
                .ok_or_else(|| {
                    invalid(format!(
                        "interrupt {interrupt} is not an entry of [interrupt] \
                         with a number below 256"
                    ))
                })
        };
        let mut devices = Vec::new();
        let entries = match table.get("device") {
            Some(toml::Value::Table(entries)) => entries.clone(),
            Some(_) => return Err(invalid("[device] must be a table")),
            None => toml::Table::new(),
        };
        for (device_name, value) in entries {
            if value.get("model").is_none() {
                continue;
            }
            let device: ProfileDevice = value.try_into().map_err(|error| {
                invalid(format!("device {device_name}: {error}"))
            })?;
            let kind = device.model.as_str();
            DeviceKind::named(kind)?.check_keys(
                device.attached_device.is_some(),
                device.sectors.is_some() || device.read_only.is_some(),
                false,
            )?;
            let interrupt_source =
                match (&device.interrupt, device.interrupt_source) {
                    (Some(_), Some(_)) => {
                        return Err(invalid(format!(
                            "device {device_name} has both interrupt and \
                             interrupt_source; keep one"
                        )));
                    }
                    (Some(interrupt), None) => {
                        Some(interrupt_number(interrupt)?)
                    }
                    (None, source) => source,
                };
            devices.push(DeviceConfiguration {
                attached_device: device
                    .attached_device
                    .unwrap_or_else(|| "none".to_owned()),
                base: device.start,
                interrupt_source,
                kind: kind.into(),
                layout: None,
                name: device_name,
                read_only: device.read_only.unwrap_or(false),
                sectors: device.sectors.unwrap_or(DEFAULT_SECTORS),
            });
        }
        let cheriot = settings.processor == "cheriot";
        let capabilities = settings
            .extensions
            .as_ref()
            .is_some_and(|names| names.iter().any(|name| name == "y"));
        let memory = settings
            .memory
            .into_iter()
            .map(|(memory_name, entry)| {
                let entry: MemoryEntry = entry.try_into().map_err(|error| {
                    invalid(format!(
                        "[emulator]: memory {memory_name}: {error}"
                    ))
                })?;
                Ok(MemoryConfiguration {
                    base: entry.start,
                    executable: entry.executable,
                    kind: entry.kind,
                    name: memory_name,
                    size: entry.length,
                    tagging: match (entry.tagged, cheriot, capabilities) {
                        (Some(false), _, _) | (None, false, false) => {
                            Tagging::None
                        }
                        (_, true, _) | (Some(true), false, false) => {
                            Tagging::Cheriot
                        }
                        (_, false, true) => Tagging::Rvy,
                    },
                })
            })
            .collect::<Result<Vec<_>, EmulatorError>>()?;
        let shared_timer = table
            .get("timer_hz")
            .and_then(toml::Value::as_integer)
            .and_then(|rate| u32::try_from(rate).ok());
        Ok(MachineDescription {
            completion: settings.completion.map(
                |completion| match completion {
                    Completion::Address(address) => {
                        CompletionPolicy::Address(address)
                    }
                    Completion::Disabled => CompletionPolicy::Disabled,
                    Completion::ElfSymbol => CompletionPolicy::ElfSymbol,
                },
            ),
            devices,
            extensions: settings.extensions,
            memory,
            name,
            processor: settings.processor,
            steps_per_tick: settings.steps_per_tick,
            timer_hz: settings
                .timer_hz
                .or(shared_timer)
                .unwrap_or(DEFAULT_TIMER_HZ),
            timing: match settings.timing {
                Timing::Elapsed => TimingPolicy::Elapsed,
                Timing::PerStep => TimingPolicy::PerStep,
            },
        })
    }
}

fn executable() -> bool {
    true
}

fn invalid(message: impl std::fmt::Display) -> EmulatorError {
    EmulatorError::Configuration(crate::program::wording::plain(
        &message.to_string(),
    ))
}

pub fn read(text: &str) -> Result<MachineConfiguration, EmulatorError> {
    if text.len() > MAX_PROFILE_BYTES {
        return Err(EmulatorError::Configuration(
            "configuration exceeds 1 MiB".into(),
        ));
    }
    MachineConfiguration::new(description(text)?)
}
