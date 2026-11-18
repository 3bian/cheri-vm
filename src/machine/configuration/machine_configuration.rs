// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::MachineDescription;
use super::default_devices::DefaultDevices;
use super::device_configuration::DeviceConfiguration;
use super::extensions::{check_extensions, extensions_for};
use super::memory_configuration::MemoryConfiguration;
use crate::hart::plain::model::Model;
use crate::hart::plain::{self, Plain};
use crate::hart::privilege::PrivilegeModel;
use crate::hart::{CheriotRv32e, Hart, Rv32Plain, cheriot_rv32e};
use crate::isa::extension::ExtensionSet;
use crate::isa::xlen::{Rv64, Word, Xlen};
use crate::machine::bus::Bus;
use crate::machine::completion::CompletionPolicy;
use crate::machine::devices::{DeviceKind, plic};
use crate::machine::error::EmulatorError;
use crate::machine::firmware::{
    load_cheriot_elf, load_elf, load_raw, load_wide_elf,
};
use crate::machine::memory::{BACKING_LIMIT, MemoryKind, MemoryRegion};
use crate::machine::timing_policy::TimingPolicy;

#[derive(Debug)]
pub struct MachineConfiguration {
    pub completion: Option<CompletionPolicy>,
    pub default_devices: DefaultDevices,
    pub devices: Vec<DeviceConfiguration>,
    pub extensions: ExtensionSet,
    pub memory: Vec<MemoryConfiguration>,
    pub name: String,
    pub processor_type: String,
    pub steps_per_tick: u32,
    pub timer_hz: u32,
    pub timing: TimingPolicy,
}

impl MachineConfiguration {
    pub(crate) fn build(
        self,
        entry: u64,
    ) -> Result<Hart<Rv32Plain>, EmulatorError> {
        self.plain(entry)
    }

    pub(super) fn build_bus(self, entry: u64) -> Result<Bus, EmulatorError> {
        let cheriot = self.is_cheriot();

        if self.name.trim().is_empty() || self.memory.is_empty() {
            return Err(EmulatorError::Configuration(
                "machine name and memory are required".into(),
            ));
        }

        check_extensions(&self.processor_type, self.extensions)?;

        let total_memory = self
            .memory
            .iter()
            .try_fold(0usize, |total, region| total.checked_add(region.size));
        let kinds = self
            .devices
            .iter()
            .map(|device| DeviceKind::named(&device.kind))
            .collect::<Result<Vec<_>, _>>()?;
        let total_storage = self
            .devices
            .iter()
            .zip(&kinds)
            .try_fold(0usize, |total, (device, kind)| {
                total.checked_add(kind.backing(device)?)
            });
        let total_backing =
            total_memory.and_then(|memory| memory.checked_add(total_storage?));

        if self.memory.len() > 64
            || self.devices.len() > 64
            || total_backing.is_none_or(|total| total > BACKING_LIMIT)
        {
            return Err(EmulatorError::Configuration(
                "at most 64 memory/device regions and 2 GiB total \
                 RAM/ROM/storage backing are supported"
                    .into(),
            ));
        }

        if !entry
            .is_multiple_of(u64::from(self.extensions.instruction_alignment()))
        {
            return Err(EmulatorError::Configuration(
                "entry must be instruction-aligned".into(),
            ));
        }

        if cheriot {
            self.validate_cheriot_memory()?;
        } else if kinds.iter().any(|kind| kind.needs_cheriot) {
            return Err(EmulatorError::Configuration(
                "the revoker model needs a CHERIoT processor: it revokes \
                 capabilities, which other processors do not have"
                    .into(),
            ));
        }

        let controller_count =
            kinds.iter().filter(|kind| kind.controls_interrupts).count();
        let mut sources = std::collections::BTreeSet::new();

        if controller_count > 1 {
            return Err(EmulatorError::Configuration(
                "only one interrupt controller is supported".into(),
            ));
        }

        let highest_source = if kinds.iter().any(|kind| kind.name == "plic") {
            plic::SOURCES
        } else {
            31
        };

        for (device, kind) in self.devices.iter().zip(&kinds) {
            if let Some(source) = device.interrupt_source {
                if !kind.routes_interrupts && !kind.controls_interrupts {
                    return Err(EmulatorError::Configuration(
                        "a 64-bit timer cannot use interrupt_source routing"
                            .into(),
                    ));
                }

                if controller_count != 1
                    || kind.controls_interrupts
                    || !(1..=highest_source).contains(&u32::from(source))
                    || !sources.insert(source)
                {
                    return Err(EmulatorError::Configuration(format!(
                        "interrupt_source must be unique, 1..\
                         {highest_source}, and routed to one configured \
                         interrupt controller"
                    )));
                }
            }
        }

        let mut bus =
            Bus::with_address_bits(if self.is_wide() { 56 } else { 32 });

        for memory in self.memory {
            if bus.memory.iter().any(|region| region.name == memory.name) {
                return Err(EmulatorError::Configuration(format!(
                    "duplicate memory region name: {}",
                    memory.name
                )));
            }

            if memory.name.is_empty() {
                return Err(EmulatorError::Configuration(
                    "memory name must not be empty".into(),
                ));
            }

            let kind = match memory.kind.as_str() {
                "ram" => MemoryKind::Ram,
                "rom" => MemoryKind::Rom,
                _ => {
                    return Err(EmulatorError::Configuration(format!(
                        "unknown memory type: {}",
                        memory.kind
                    )));
                }
            };
            let mut region = MemoryRegion::new(
                memory.name,
                memory.base,
                memory.size,
                kind,
                memory.tagging,
            )?;
            region.executable = memory.executable;
            bus.add_memory(region)?;
        }

        for device in self.devices {
            if device.base % 4 != 0 {
                return Err(EmulatorError::Configuration(format!(
                    "device {} base must be 4-byte aligned",
                    device.name
                )));
            }

            if device.name.is_empty() {
                return Err(EmulatorError::Configuration(
                    "device name must not be empty".into(),
                ));
            }

            let kind = DeviceKind::named(&device.kind)?;
            kind.check_keys(
                device.attached_device != "none",
                device.sectors != 16 || device.read_only,
                device.layout.is_some(),
            )?;
            let instance = (kind.make)(&device)?;
            let size = kind.size;
            let device_name = device.name.clone();
            bus.add_device(device.name, device.base, size, instance)?;
            bus.set_interrupt_source(&device_name, device.interrupt_source)?;
        }

        Ok(bus)
    }

    pub(crate) fn build_cheriot(
        self,
        firmware: &[u8],
    ) -> Result<Hart<CheriotRv32e>, EmulatorError> {
        if !self.is_cheriot() {
            return Err(EmulatorError::Configuration(
                "build_cheriot requires the cheriot processor type".into(),
            ));
        }

        crate::machine::timing_policy::validate_timer_hz(self.timer_hz)?;
        crate::machine::timing_policy::validate_step_period(
            self.timing,
            self.steps_per_tick,
        )?;

        let timing = self.timing;
        let steps_per_tick = self.steps_per_tick;
        let timer_hz = self.timer_hz;
        let defaults = self.default_devices;
        let completion = self.completion.unwrap_or(CompletionPolicy::ElfSymbol);
        let mut bus = self.build_bus(0)?;
        defaults.install(&mut bus)?;
        let host_interface = completion.build(&bus, Some(firmware))?;
        let entry = load_cheriot_elf(&mut bus, firmware)?;
        let mut system = Hart::new(cheriot_rv32e::Processor::new(entry), bus);
        system.host_interface = host_interface;
        system.set_timer_hz(timer_hz)?;
        system.timing = timing;
        system.set_steps_per_tick(steps_per_tick)?;

        Ok(system)
    }

    pub(crate) fn build_cheriot_raw(
        self,
        address: u64,
        firmware: &[u8],
    ) -> Result<Hart<CheriotRv32e>, EmulatorError> {
        if !self.is_cheriot() {
            return Err(EmulatorError::Configuration(
                "build_cheriot_raw requires the cheriot processor type".into(),
            ));
        }

        crate::machine::timing_policy::validate_timer_hz(self.timer_hz)?;
        crate::machine::timing_policy::validate_step_period(
            self.timing,
            self.steps_per_tick,
        )?;

        let timing = self.timing;
        let steps_per_tick = self.steps_per_tick;
        let timer_hz = self.timer_hz;
        let defaults = self.default_devices;
        let completion = self.completion.unwrap_or(CompletionPolicy::Disabled);
        let mut bus = self.build_bus(address)?;
        defaults.install(&mut bus)?;
        crate::machine::firmware::validate_raw_entry(
            &bus, address, firmware, 2,
        )?;
        let host_interface = completion.build(&bus, None)?;
        load_raw(&mut bus, address, firmware)?;
        let entry = u32::try_from(address).map_err(|_| {
            EmulatorError::Configuration(
                "a CHERIoT machine has addresses below 4 GiB".into(),
            )
        })?;
        let mut system = Hart::new(cheriot_rv32e::Processor::new(entry), bus);
        system.host_interface = host_interface;
        system.set_timer_hz(timer_hz)?;
        system.timing = timing;
        system.set_steps_per_tick(steps_per_tick)?;

        Ok(system)
    }

    pub(crate) fn build_elf(
        self,
        firmware: &[u8],
    ) -> Result<Hart<Rv32Plain>, EmulatorError> {
        self.plain_elf(firmware, |bus, firmware, extensions| {
            load_elf(bus, firmware, extensions).map(u64::from)
        })
    }

    pub(crate) fn build_raw(
        self,
        address: u64,
        firmware: &[u8],
    ) -> Result<Hart<Rv32Plain>, EmulatorError> {
        Self::with_raw(self.build(address)?, address, firmware)
    }

    pub(crate) fn build_wide_machine<P: PrivilegeModel, M: Model<Rv64>>(
        self,
        elf: bool,
        address: u64,
        firmware: &[u8],
    ) -> Result<Hart<Plain<Rv64, P, M>>, EmulatorError> {
        if elf {
            self.plain_elf(firmware, load_wide_elf)
        } else {
            Self::with_raw(self.plain(address)?, address, firmware)
        }
    }

    pub(crate) fn has_capabilities(&self) -> bool {
        self.extensions.contains(ExtensionSet::Y)
    }

    pub(crate) fn is_cheriot(&self) -> bool {
        self.processor_type == "cheriot"
    }

    pub(crate) fn is_supervisor(&self) -> bool {
        self.extensions.contains(ExtensionSet::S)
    }

    pub(crate) fn is_wide(&self) -> bool {
        self.processor_type == "rv64"
    }

    pub fn new(document: MachineDescription) -> Result<Self, EmulatorError> {
        let extensions = extensions_for(
            &document.processor,
            document.extensions.as_deref(),
        )?;

        crate::machine::timing_policy::validate_timer_hz(document.timer_hz)?;
        crate::machine::timing_policy::validate_step_period(
            document.timing,
            document.steps_per_tick,
        )?;
        let default_devices = if document.processor == "cheriot" {
            DefaultDevices::CheriotCompatibility
        } else {
            DefaultDevices::Configured
        };
        Ok(Self {
            completion: document.completion,
            default_devices,
            name: document.name,
            processor_type: document.processor,
            extensions,
            memory: document.memory,
            devices: document.devices,
            steps_per_tick: document.steps_per_tick,
            timer_hz: document.timer_hz,
            timing: document.timing,
        })
    }

    pub(super) fn plain<X: Xlen, P: PrivilegeModel, M: Model<X>>(
        self,
        entry: u64,
    ) -> Result<Hart<Plain<X, P, M>>, EmulatorError> {
        if self.is_cheriot() {
            return Err(EmulatorError::Configuration(
                "CHERIoT requires build_cheriot or build_cheriot_raw".into(),
            ));
        }
        if X::BITS == 32 && entry > u64::from(u32::MAX) {
            return Err(EmulatorError::Configuration(
                "a 32-bit machine has addresses below 4 GiB".into(),
            ));
        }
        if self.is_supervisor() != P::SUPERVISOR {
            return Err(EmulatorError::Configuration(format!(
                "the {} profile {} supervisor and user modes",
                self.name,
                if P::SUPERVISOR { "lacks" } else { "has" }
            )));
        }
        if self.has_capabilities() != M::CAPABILITIES {
            return Err(EmulatorError::Configuration(format!(
                "the {} profile {} capabilities",
                self.name,
                if M::CAPABILITIES { "lacks" } else { "has" }
            )));
        }
        if self.is_wide() != (X::BITS == 64) {
            return Err(EmulatorError::Configuration(format!(
                "the {} processor is not a {}-bit one",
                self.processor_type,
                X::BITS
            )));
        }

        crate::machine::timing_policy::validate_timer_hz(self.timer_hz)?;
        crate::machine::timing_policy::validate_step_period(
            self.timing,
            self.steps_per_tick,
        )?;

        let extensions = self.extensions;
        let timing = self.timing;
        let steps_per_tick = self.steps_per_tick;
        let timer_hz = self.timer_hz;
        let completion = self.completion.unwrap_or(CompletionPolicy::Disabled);
        let defaults = self.default_devices;
        let mut bus = self.build_bus(entry)?;
        defaults.install(&mut bus)?;
        let host_interface = completion.build(&bus, None)?;
        let entry = X::Word::from_u64(entry);
        let mut computer =
            Hart::new(plain::Processor::new(extensions, entry), bus);
        computer.set_timer_hz(timer_hz)?;
        computer.host_interface = host_interface;
        computer.timing = timing;
        computer.set_steps_per_tick(steps_per_tick)?;

        Ok(computer)
    }

    pub(super) fn plain_elf<X: Xlen, P: PrivilegeModel, M: Model<X>>(
        mut self,
        firmware: &[u8],
        load: impl FnOnce(
            &mut Bus,
            &[u8],
            ExtensionSet,
        ) -> Result<u64, EmulatorError>,
    ) -> Result<Hart<Plain<X, P, M>>, EmulatorError> {
        let completion = self.completion.unwrap_or(CompletionPolicy::Disabled);
        self.completion = Some(CompletionPolicy::Disabled);
        let mut computer = self.plain::<X, P, M>(0)?;
        let interface = completion.build(&computer.bus, Some(firmware))?;
        let entry =
            load(&mut computer.bus, firmware, computer.processor.extensions)?;
        computer.host_interface = interface;
        computer.processor.program_counter = X::Word::from_u64(entry);
        Ok(computer)
    }

    pub(super) fn with_raw<X: Xlen, P: PrivilegeModel, M: Model<X>>(
        mut computer: Hart<Plain<X, P, M>>,
        address: u64,
        firmware: &[u8],
    ) -> Result<Hart<Plain<X, P, M>>, EmulatorError> {
        crate::machine::firmware::validate_raw_entry(
            &computer.bus,
            address,
            firmware,
            computer.processor.extensions.instruction_alignment(),
        )?;
        load_raw(&mut computer.bus, address, firmware)?;
        Ok(computer)
    }
}
