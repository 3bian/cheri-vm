// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::{
    device_settings,
    options::ethernet::EthernetOptions,
    options::gpio::GpioOptions,
    options::host::HostOptions,
    options::i2c::I2cOptions,
    options::spi::SpiOptions,
    options::uart::UartOptions,
    options::{Options, parse_unsigned},
    output::OutputPresentation,
    register_selection::RegisterSelection,
};
use super::{
    ExitLine, Firmware, PeripheralSettings, Presentation, RegisterDump,
    StatsReport,
};
use crate::hart::execution::TrapPolicy;
use crate::machine::EmulatorError;
use crate::machine::completion::CompletionPolicy;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug)]
pub struct MachineRequest {
    pub firmware: Firmware,
    pub profile: PathBuf,
    pub profile_text: Option<String>,
    pub stats: Option<StatsReport>,
    pub breakpoints: Vec<u64>,
    pub completion: Option<CompletionPolicy>,
    pub peripherals: PeripheralSettings,
    pub memory: Option<String>,
    pub output: Option<Presentation>,
    pub registers: Option<RegisterDump>,
    pub exit_line: ExitLine,
    pub steps: Option<u64>,
    pub trace: bool,
    pub trap_policy: Option<TrapPolicy>,
    pub debugger: Option<SocketAddr>,
    pub debugger_client: Option<PathBuf>,
    pub debugger_raw_output: bool,
}

impl MachineRequest {
    pub fn new(firmware: Firmware, profile: PathBuf) -> Self {
        Self {
            firmware,
            profile,
            profile_text: None,
            stats: None,
            breakpoints: vec![],
            completion: None,
            peripherals: PeripheralSettings::default(),
            memory: None,
            output: None,
            registers: None,
            exit_line: ExitLine::Shown,
            steps: None,
            trace: false,
            trap_policy: None,
            debugger: None,
            debugger_client: None,
            debugger_raw_output: false,
        }
    }
    pub(crate) fn validate(self) -> Result<Options, EmulatorError> {
        let path = |p: PathBuf| {
            p.into_os_string().into_string().map_err(|_| {
                EmulatorError::Configuration("paths must be UTF-8".into())
            })
        };
        let (firmware_path, firmware_is_elf, raw_address) = match self.firmware
        {
            Firmware::Elf(p) => (path(p)?, true, 0),
            Firmware::Raw { path: p, address } => (path(p)?, false, address),
        };
        let mut host = HostOptions::default();
        for v in self.peripherals.ethernet {
            host.ethernet.push(EthernetOptions::parse(&v)?);
        }
        for v in self.peripherals.gpio {
            host.gpio.push(GpioOptions::parse(&v)?);
        }
        for v in self.peripherals.i2c {
            host.i2c.push(I2cOptions::parse(&v)?);
        }
        for v in self.peripherals.spi {
            host.spi.push(SpiOptions::parse(&v)?);
        }
        for v in self.peripherals.uart {
            host.uart.push(UartOptions::parse(&v)?);
        }
        if !self.peripherals.web.host.is_loopback() {
            return Err(EmulatorError::Configuration(
                "--host must be a loopback IP address".into(),
            ));
        }
        host.web = self.peripherals.web;
        if host.count() > 64 {
            return Err(EmulatorError::Configuration(
                "at most 64 host bindings are supported".into(),
            ));
        }
        let memory_dump = self
            .memory
            .map(|v| {
                let settings = device_settings::parse(
                    &v,
                    "--memory",
                    &["address", "length"],
                )?;
                let address = settings.get("address").ok_or_else(|| {
                    EmulatorError::Configuration(
                        "--memory requires address=ADDRESS".into(),
                    )
                })?;
                let start = parse_unsigned(address, "memory address")?;
                let count = settings.get("length").map_or(Ok(16), |v| {
                    parse_unsigned(v, "memory dump length")
                })?;
                if !(1..=256).contains(&count) {
                    return Err(EmulatorError::Configuration(
                        "memory dump length must be 1..256".into(),
                    ));
                }
                if start.checked_add(count).is_none() {
                    return Err(EmulatorError::Configuration(
                        "memory dump address overflow".into(),
                    ));
                }
                Ok((start, count as usize))
            })
            .transpose()?;
        let registers = self
            .registers
            .map(|v| {
                let RegisterDump::Selection(v) = v;
                let name = v.strip_prefix("names=").unwrap_or(&v);
                RegisterSelection::parse(name)
            })
            .transpose()?;
        let output = self.output.map(|p| match p {
            Presentation::Raw => OutputPresentation::Raw,
            Presentation::Labeled => OutputPresentation::Labeled,
            Presentation::Escaped => OutputPresentation::Escaped,
        });
        if let Some(address) = self.debugger {
            if !address.ip().is_loopback() {
                return Err(EmulatorError::Configuration(
                    "--host must be a loopback IP address".into(),
                ));
            }
            if !cfg!(all(feature = "gdb", unix)) {
                return Err(EmulatorError::Configuration(
                    "debug is not part of this build: it needs the gdb \
                     feature and a Unix system"
                        .into(),
                ));
            }
            if output.is_some()
                || self.trace
                || registers.is_some()
                || memory_dump.is_some()
                || !self.breakpoints.is_empty()
                || self.trap_policy.is_some()
                || host.has_hardware()
                || host.has_socket()
            {
                return Err(EmulatorError::Configuration(
                    "debug does not take --console-format, --trace, \
                     --registers, --memory, --breakpoint, --trap-policy, live \
                     Ethernet/GPIO/SPI/I2C or a WebSocket; use debugger commands \
                     instead"
                        .into(),
                ));
            }
        }
        Ok(Options {
            breakpoint: self.breakpoints,
            completion: self.completion,
            firmware_is_elf,
            firmware_path,
            gdb: self.debugger,
            gdb_client: self.debugger_client,
            gdb_raw_output: self.debugger_raw_output,
            host,
            memory_dump,
            output,
            profile_path: path(self.profile)?,
            profile_text: self.profile_text,
            stats: self.stats,
            raw_address,
            registers,
            exit_line: self.exit_line,
            steps: self.steps,
            trace: self.trace,
            trap_policy: self.trap_policy,
        })
    }
}
