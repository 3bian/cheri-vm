// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::run;
use crate::hart::execution::TrapPolicy;
use crate::hart::plain::Model;
use crate::hart::privilege::PrivilegeModel;
use crate::isa::xlen::Rv64;
use crate::machine::EmulatorError;
use crate::machine::MachineConfiguration;
use crate::runtime::options::run::RunOptions;

pub(super) struct Launch<'a> {
    pub(super) listening: Option<&'a dyn Fn(std::net::SocketAddr)>,
    pub(super) host: crate::runtime::options::host::HostOptions,
    pub(super) profile_name: String,
    pub(super) gdb: Option<std::net::SocketAddr>,
    #[cfg(all(feature = "gdb", unix))]
    pub(super) session: (Option<u64>, bool, Option<run::debugger::GdbClient>),
}

impl Launch<'_> {
    pub(super) fn run<A: run::hart::Runnable>(
        self,
        mut hart: crate::hart::Hart<A>,
        run_options: impl FnOnce(crate::runtime::host_bindings::Hosts) -> RunOptions,
    ) -> Result<bool, EmulatorError> {
        let hosts = crate::runtime::host_bindings::connect(
            &mut hart.bus,
            self.host,
            &self.profile_name,
            self.gdb.is_some(),
        )?;
        #[cfg(all(feature = "gdb", unix))]
        if let Some(address) = self.gdb {
            let (steps, raw, client) = self.session;
            if let Some(listening) = self.listening {
                listening(address);
            }
            run::debugger::run(
                hart,
                address,
                steps,
                raw,
                hosts.0,
                client.as_ref(),
            )?;
            return Ok(false);
        }
        #[cfg(not(all(feature = "gdb", unix)))]
        let _ = self.listening;
        run::hart::run_with_options(hart, run_options(hosts))?;
        Ok(true)
    }

    pub(super) fn run_wide<P: PrivilegeModel, M: Model<Rv64>>(
        self,
        (configuration, (elf, address, firmware), trap_policy): (
            MachineConfiguration,
            (bool, u64, &[u8]),
            TrapPolicy,
        ),
        run_options: impl FnOnce(crate::runtime::host_bindings::Hosts) -> RunOptions,
    ) -> Result<bool, EmulatorError>
    where
        crate::hart::plain::Plain<Rv64, P, M>: run::hart::Runnable,
    {
        let mut hart =
            configuration.build_wide_machine::<P, M>(elf, address, firmware)?;
        hart.trap_policy = trap_policy;
        self.run(hart, run_options)
    }
}
