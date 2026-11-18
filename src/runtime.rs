// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod adapter_machine;
mod launch;

pub use adapter_machine::AdapterMachine;
use launch::Launch;

use crate::debug::adapter::System;

use crate::debug::architecture::DebugArchitecture;

use crate::hart::execution::TrapPolicy;

use crate::hart::plain::debug::DebugModel;

use crate::hart::plain::Flat;

use crate::hart::privilege::{MachineOnly, PrivilegeModel, Supervisor};

use crate::isa::pointer::Rvy;

use crate::isa::xlen::Rv64;

use crate::machine::EmulatorError;

use crate::machine::MachineConfiguration;

use crate::runtime::input::{MAX_FIRMWARE_BYTES, read_file};

use crate::runtime::options::run::RunOptions;

use crate::runtime::output::OutputPresentation;

use options::Options;

mod device_settings;

pub(crate) mod host_bindings;

pub mod host_output;

pub(crate) mod input;

pub(crate) mod insights;

pub(crate) mod options;

pub(crate) mod output;

pub(crate) mod register;

pub(crate) mod register_selection;

pub mod request;

pub(crate) mod run;

pub(crate) mod runtime_io;

pub(crate) mod uart;

pub fn adapter_machine(
    request: crate::runtime::request::MachineRequest,
) -> Result<AdapterMachine, EmulatorError> {
    let options = request.validate()?;
    let profile = match options.profile_text {
        Some(text) => text.into_bytes(),
        None => read_file(
            options.profile_path,
            crate::profile::reader::MAX_PROFILE_BYTES,
            "profile",
        )?,
    };
    let profile = std::str::from_utf8(&profile).map_err(|error| {
        EmulatorError::Configuration(format!("profile is not UTF-8: {error}"))
    })?;
    let mut configuration = crate::profile::reader::read(profile)?;
    if options.completion.is_some() {
        configuration.completion = options.completion;
    }
    let firmware =
        read_file(options.firmware_path, MAX_FIRMWARE_BYTES, "firmware")?;
    if options.host.has_socket() {
        return Err(EmulatorError::Configuration(
            "the debug adapter does not take a WebSocket".into(),
        ));
    }
    let name = configuration.name.clone();
    let (elf, address, host) =
        (options.firmware_is_elf, options.raw_address, options.host);
    let (system, uart) = if configuration.is_cheriot() {
        let hart = if elf {
            configuration.build_cheriot(&firmware)?
        } else {
            configuration.build_cheriot_raw(address, &firmware)?
        };
        adapter_system(hart, host, &name)?
    } else if configuration.is_wide() {
        let image = (elf, address, firmware.as_slice());
        match (
            configuration.has_capabilities(),
            configuration.is_supervisor(),
        ) {
            (true, true) => wide_adapter::<Supervisor, Rvy>(
                configuration,
                image,
                host,
                &name,
            )?,
            (true, false) => wide_adapter::<MachineOnly, Rvy>(
                configuration,
                image,
                host,
                &name,
            )?,
            (false, true) => wide_adapter::<Supervisor, Flat>(
                configuration,
                image,
                host,
                &name,
            )?,
            (false, false) => wide_adapter::<MachineOnly, Flat>(
                configuration,
                image,
                host,
                &name,
            )?,
        }
    } else {
        let hart = if elf {
            configuration.build_elf(&firmware)?
        } else {
            configuration.build_raw(address, &firmware)?
        };
        adapter_system(hart, host, &name)?
    };
    Ok(AdapterMachine {
        system,
        firmware,
        _uart: uart,
    })
}

fn adapter_system<A: DebugArchitecture>(
    mut hart: crate::hart::Hart<A>,
    host: crate::runtime::options::host::HostOptions,
    name: &str,
) -> Result<
    (
        Box<dyn System>,
        crate::runtime::uart::connections::UartConnections,
    ),
    EmulatorError,
>
where
    crate::hart::Hart<A>: 'static,
{
    hart.trap_policy = TrapPolicy::Vector;
    let (uart, _, _) = crate::runtime::host_bindings::connect(
        &mut hart.bus,
        host,
        name,
        true,
    )?;
    hart.pause_host_clock();
    Ok((Box::new(hart), uart))
}

pub fn debug_request(
    request: request::MachineRequest,
    listening: &dyn Fn(std::net::SocketAddr),
) -> Result<Option<String>, EmulatorError> {
    execute_options(request.validate()?, Some(listening))
}

fn execute_options(
    options: Options,
    listening: Option<&dyn Fn(std::net::SocketAddr)>,
) -> Result<Option<String>, EmulatorError> {
    let Options {
        breakpoint,
        completion,
        firmware_is_elf,
        firmware_path,
        gdb,
        gdb_client,
        gdb_raw_output,
        host,
        memory_dump,
        output,
        profile_path,
        profile_text,
        raw_address,
        stats,
        exit_line,
        registers,
        steps,
        trace,
        trap_policy,
    } = options;
    let profile = match profile_text {
        Some(text) => text.into_bytes(),
        None => read_file(
            profile_path,
            crate::profile::reader::MAX_PROFILE_BYTES,
            "profile",
        )?,
    };
    let profile = std::str::from_utf8(&profile).map_err(|error| {
        EmulatorError::Configuration(format!("profile is not UTF-8: {error}"))
    })?;
    let mut configuration = crate::profile::reader::read(profile)?;
    if let Some(selection) = &registers {
        selection
            .validate(configuration.is_cheriot(), configuration.extensions)?;
    }
    if completion.is_some() {
        configuration.completion = completion;
    }
    let is_cheriot = configuration.is_cheriot();
    let trap_policy = trap_policy.unwrap_or(if is_cheriot {
        TrapPolicy::Vector
    } else {
        TrapPolicy::Stop
    });
    let presentation = output.unwrap_or(if is_cheriot {
        OutputPresentation::Compatibility
    } else {
        OutputPresentation::Labeled
    });
    let firmware = read_file(&firmware_path, MAX_FIRMWARE_BYTES, "firmware")?;
    #[cfg(all(feature = "gdb", unix))]
    let client = gdb_client.filter(|_| firmware_is_elf).map(|executable| {
        run::debugger::GdbClient {
            executable,
            firmware: firmware_path.into(),
        }
    });
    let insights = match &stats {
        Some(_) if !firmware_is_elf => {
            return Err(EmulatorError::Configuration(
                "--stats needs an ELF image, which names the code sections"
                    .into(),
            ));
        }
        Some(_) => Some(std::rc::Rc::new(std::cell::RefCell::new(
            insights::Insights::from_elf(&firmware).ok_or_else(|| {
                EmulatorError::Configuration(
                    "--stats found no code sections in the ELF".into(),
                )
            })?,
        ))),
        None => None,
    };
    let collected = insights.clone();
    let profile_name = configuration.name.clone();
    let run_options = |(uart, websocket, web)| RunOptions {
        insights,
        uart,
        web,
        websocket,
        breakpoint,
        memory_dump,
        presentation,
        exit_line,
        registers,
        steps,
        trace,
        trap_policy,
    };
    let launch = Launch {
        listening,
        host,
        profile_name,
        gdb,
        #[cfg(all(feature = "gdb", unix))]
        session: (steps, gdb_raw_output, client),
    };
    #[cfg(not(all(feature = "gdb", unix)))]
    let _ = (gdb_client, gdb_raw_output);
    if is_cheriot {
        let hart = if firmware_is_elf {
            configuration.build_cheriot(&firmware)?
        } else {
            configuration.build_cheriot_raw(raw_address, &firmware)?
        };
        if !launch.run(hart, run_options)? {
            return Ok(None);
        }
    } else if configuration.is_wide() {
        let image = (firmware_is_elf, raw_address, firmware.as_slice());
        let capabilities = configuration.has_capabilities();
        let supervisor = configuration.is_supervisor();
        let hart = (configuration, image, trap_policy);
        let finished = match (capabilities, supervisor) {
            (true, true) => {
                launch.run_wide::<Supervisor, Rvy>(hart, run_options)?
            }
            (true, false) => {
                launch.run_wide::<MachineOnly, Rvy>(hart, run_options)?
            }
            (false, true) => {
                launch.run_wide::<Supervisor, Flat>(hart, run_options)?
            }
            (false, false) => {
                launch.run_wide::<MachineOnly, Flat>(hart, run_options)?
            }
        };
        if !finished {
            return Ok(None);
        }
    } else {
        let mut hart = if firmware_is_elf {
            configuration.build_elf(&firmware)?
        } else {
            configuration.build_raw(raw_address, &firmware)?
        };
        hart.trap_policy = trap_policy;
        if !launch.run(hart, run_options)? {
            return Ok(None);
        }
    }
    report_stats(collected, stats)
}

pub fn execute_request(
    request: request::MachineRequest,
) -> Result<Option<String>, EmulatorError> {
    execute_options(request.validate()?, None)
}

fn report_stats(
    insights: Option<std::rc::Rc<std::cell::RefCell<insights::Insights>>>,
    destination: Option<request::StatsReport>,
) -> Result<Option<String>, EmulatorError> {
    let (Some(insights), Some(destination)) = (insights, destination) else {
        return Ok(None);
    };
    let report = insights.borrow().report();
    let mut rendered =
        crate::program::output::render(&report, destination.format)
            .map_err(|error| EmulatorError::Configuration(error.to_string()))?;
    if !rendered.ends_with(b"\n") {
        rendered.push(b'\n');
    }
    match destination.file {
        Some(path) => {
            std::fs::write(&path, rendered)
                .map_err(|error| EmulatorError::host_file(&path, &error))?;
            Ok(None)
        }
        None => Ok(Some(String::from_utf8_lossy(&rendered).into_owned())),
    }
}

fn wide_adapter<P: PrivilegeModel, M: DebugModel<Rv64>>(
    configuration: MachineConfiguration,
    (elf, address, firmware): (bool, u64, &[u8]),
    host: crate::runtime::options::host::HostOptions,
    name: &str,
) -> Result<
    (
        Box<dyn System>,
        crate::runtime::uart::connections::UartConnections,
    ),
    EmulatorError,
> {
    adapter_system(
        configuration.build_wide_machine::<P, M>(elf, address, firmware)?,
        host,
        name,
    )
}
