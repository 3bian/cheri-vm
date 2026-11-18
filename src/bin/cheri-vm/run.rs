// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::cli::STATS_FORMATS;
use crate::cli::{RunArgs, RunCommandArgs};
use crate::error::{AppError, Result, require_utf8_path};
use cheri_vm::hart::execution::TrapPolicy;
use cheri_vm::machine::completion::CompletionPolicy;
use cheri_vm::profile::ProfileDocument;
use cheri_vm::program::command::Context;
use cheri_vm::program::commands::{
    APPLICATION, ETHERNET_VARIABLE, GPIO_VARIABLE, I2C_VARIABLE,
    PROFILE_VARIABLE, SPI_VARIABLE, UART_VARIABLE,
};
use cheri_vm::program::report;
use cheri_vm::program::settings::read_environment;
use cheri_vm::runtime::request::{
    self, AddressKind, ExitLine, Firmware, MachineRequest, PeripheralSettings,
    Presentation, RegisterDump, StatsReport, WebSettings,
};

fn bindings(given: Vec<String>, variable: &str) -> Result<Vec<String>> {
    if !given.is_empty() {
        return Ok(given);
    }
    Ok(read_environment(variable)?.into_iter().collect())
}

pub(crate) fn handle(e: RunCommandArgs, context: &Context) -> Result<u8> {
    let mut machine = request(e.run)?;
    if e.quiet {
        machine.exit_line = ExitLine::Hidden;
    }
    let report =
        cheri_vm::runtime::execute_request(machine).map_err(invalid)?;
    if let Some(report) = report {
        context.shell.stdout(&format!("\n{report}\n"));
    }
    Ok(0)
}

fn invalid(e: impl std::fmt::Display) -> AppError {
    AppError::Message(e.to_string())
}

fn machine_request(
    e: RunArgs,
    profile: &ProfileDocument,
) -> Result<MachineRequest> {
    require_utf8_path(&profile.file)?;
    let Some(path) = e.file else {
        return Err(AppError::Message(format!(
            "name the firmware to run: {APPLICATION} run FIRMWARE --profile \
             NAME|FILE"
        )));
    };
    let firmware = match e.address {
        None => Firmware::Elf(path),
        Some(address) => Firmware::Raw {
            path,
            address: request::address(&address, AddressKind::Raw)
                .map_err(invalid)?,
        },
    };
    let mut request = MachineRequest::new(firmware, profile.file.clone());
    request.profile_text = Some(profile.to_toml()?);
    request.breakpoints = e
        .breakpoint
        .iter()
        .map(|v| request::address(v, AddressKind::Breakpoint).map_err(invalid))
        .collect::<Result<_>>()?;
    request.completion = e
        .completion
        .map(|v| match v.as_str() {
            "disabled" => Ok(CompletionPolicy::Disabled),
            "elf-symbol" => Ok(CompletionPolicy::ElfSymbol),
            word if !word.starts_with(|first: char| first.is_ascii_digit()) => {
                Err(AppError::Message(format!(
                    "--completion must be disabled, elf-symbol or an address: \
                     {word}"
                )))
            }
            _ => request::address(&v, AddressKind::Completion)
                .map(CompletionPolicy::Address)
                .map_err(invalid),
        })
        .transpose()?;
    let host: std::net::IpAddr = e.host.parse().map_err(|_| {
        AppError::Message("--host must be a loopback IP address".into())
    })?;
    request.peripherals = PeripheralSettings {
        web: WebSettings {
            host,
            port: e.web_port,
            root: e.web_root,
        },
        ethernet: bindings(e.ethernet, ETHERNET_VARIABLE)?,
        gpio: bindings(e.gpio, GPIO_VARIABLE)?,
        i2c: bindings(e.i2c, I2C_VARIABLE)?,
        spi: bindings(e.spi, SPI_VARIABLE)?,
        uart: bindings(e.uart, UART_VARIABLE)?,
    };
    request.memory = e.memory;
    request.registers = e.registers.map(RegisterDump::Selection);
    request.steps = e.steps;
    request.stats = e.stats.then(|| StatsReport {
        format: report::format(e.format, e.output.as_deref(), STATS_FORMATS),
        file: e.output,
    });
    request.trace = e.trace;
    request.output = e
        .console_format
        .map(|s| match s.as_str() {
            "raw" => Ok(Presentation::Raw),
            "labeled" => Ok(Presentation::Labeled),
            "escaped" => Ok(Presentation::Escaped),
            _ => Err(AppError::Message("invalid console format".into())),
        })
        .transpose()?;
    request.trap_policy = e
        .trap_policy
        .map(|s| match s.as_str() {
            "stop" => Ok(TrapPolicy::Stop),
            "vector" => Ok(TrapPolicy::Vector),
            _ => Err(AppError::Message("invalid trap policy".into())),
        })
        .transpose()?;
    Ok(request)
}

pub(crate) fn request(e: RunArgs) -> Result<MachineRequest> {
    if let Some(path) = e.file.as_deref() {
        require_utf8_path(path)?;
    }
    if let Some(address) = &e.address {
        request::address(address, AddressKind::Raw).map_err(invalid)?;
    }
    let profile = selected_profile(&e)?;
    if !profile.has_machine() {
        return Err(AppError::Message(format!(
            "profile {} has no [emulator] table, so {APPLICATION} cannot run \
             it",
            profile.name
        )));
    }
    machine_request(e, &profile)
}

fn selected_profile(e: &RunArgs) -> Result<ProfileDocument> {
    let selector = match &e.profile {
        Some(selector) => selector.clone(),
        None => read_environment(PROFILE_VARIABLE)?.ok_or_else(|| {
            AppError::Message(format!(
                "no profile selected: pass --profile NAME|FILE; `{APPLICATION} \
                 profile --list` names the machines built in"
            ))
        })?,
    };
    Ok(cheri_vm::profile::load(&selector)?)
}
