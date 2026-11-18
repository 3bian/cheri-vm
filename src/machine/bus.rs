// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::error::HostError;
use std::any::TypeId;
use std::cell::OnceCell;

use crate::machine::devices::timer64::Timer64;
use crate::machine::devices::timer64::registers::Register as TimerRegister;
use crate::machine::devices::{Device, InterruptLine};
use crate::machine::error::EmulatorError;
use crate::machine::memory::{BACKING_LIMIT, MemoryRegion};

mod device_indices;
mod device_mapping;
mod interrupts;
mod machine_time;
mod memory_port;

use device_indices::DeviceIndices;

pub(crate) use device_mapping::DeviceMapping;
pub(crate) use memory_port::MemoryPort;

pub struct Bus {
    address_limit: u64,
    device_indices: OnceCell<DeviceIndices>,
    pub(crate) devices: Vec<DeviceMapping>,
    pub(crate) memory: Vec<MemoryRegion>,
    pending_machine_time: Option<u64>,
    possibly_modified_timer: Option<usize>,
    devices_dirty: bool,
    interrupt_bits: u32,
    host_errors_clean: std::cell::Cell<bool>,
    host_wait: Option<std::rc::Rc<std::cell::Cell<std::time::Duration>>>,
}

impl Bus {
    pub(crate) fn add_device(
        &mut self,
        name: String,
        base: u64,
        size: u32,
        mut device: Box<dyn Device>,
    ) -> Result<(), EmulatorError> {
        if self.devices.len() >= 64 {
            return Err(EmulatorError::Configuration(
                "at most 64 device regions are supported".into(),
            ));
        }
        self.check_region(base, size as u64, &name)?;

        if self.devices.iter().any(|mapping| mapping.name == name) {
            return Err(EmulatorError::Configuration(format!(
                "duplicate device name {name}"
            )));
        }

        self.touch();
        self.collect_timer_write();
        let written_time = device
            .downcast_mut::<Timer64>()
            .and_then(Timer64::take_written_time);
        self.device_indices.take();
        self.devices.push(DeviceMapping {
            name,
            base,
            size,
            device,
            interrupt_source: None,
        });
        if let Some(time) = written_time {
            self.set_machine_time(time);
        }

        Ok(())
    }

    pub(crate) fn add_memory(
        &mut self,
        region: MemoryRegion,
    ) -> Result<(), EmulatorError> {
        if region.tagging().granule_bytes().is_some_and(|granule| {
            !(region.base as usize).is_multiple_of(granule)
        }) {
            return Err(EmulatorError::Configuration(
                "tagged memory base must preserve capability granule alignment"
                    .into(),
            ));
        }
        let total = self.memory.iter().map(MemoryRegion::size).sum::<usize>();
        if self.memory.len() >= 64
            || total
                .checked_add(region.size())
                .is_none_or(|size| size > BACKING_LIMIT)
        {
            return Err(EmulatorError::Configuration(
                "at most 64 memory regions and 2 GiB memory backing are \
                 supported"
                    .into(),
            ));
        }
        if self.memory.iter().any(|memory| memory.name == region.name) {
            return Err(EmulatorError::Configuration(format!(
                "duplicate memory name {}",
                region.name
            )));
        }
        self.check_region(region.base, region.size() as u64, &region.name)?;
        self.memory.push(region);

        Ok(())
    }

    #[inline]
    pub(crate) fn check_host_errors(&self) -> Result<(), EmulatorError> {
        if self.host_errors_clean.get() {
            return Ok(());
        }
        self.scan_host_errors()
    }

    fn check_region(
        &self,
        base: u64,
        size: u64,
        name: &str,
    ) -> Result<(), EmulatorError> {
        if name.trim().is_empty()
            || name.len() > 4096
            || size == 0
            || base
                .checked_add(size)
                .is_none_or(|end| end > self.address_limit)
        {
            return Err(EmulatorError::Configuration(format!(
                "invalid region {name}"
            )));
        }

        let end = base + size;
        let collides = |other_base: u64, other_size: u64| {
            base < other_base + other_size && other_base < end
        };
        if self
            .memory
            .iter()
            .any(|region| collides(region.base, region.size() as u64))
            || self
                .devices
                .iter()
                .any(|mapping| collides(mapping.base, mapping.size as u64))
        {
            return Err(EmulatorError::Configuration(format!(
                "address region {name} overlaps an existing region"
            )));
        }

        Ok(())
    }

    pub(crate) fn device<T: 'static>(&self, name: &str) -> Option<&T> {
        self.devices
            .iter()
            .find(|mapping| mapping.name == name)
            .and_then(|mapping| mapping.device.downcast_ref::<T>())
    }

    pub(crate) fn device_mut<T: 'static>(
        &mut self,
        name: &str,
    ) -> Option<&mut T> {
        let index = self
            .devices
            .iter()
            .position(|mapping| mapping.name == name)?;
        self.touch();
        self.collect_timer_write();
        if TypeId::of::<T>() == TypeId::of::<Timer64>()
            && self.devices[index].device.is::<Timer64>()
        {
            self.possibly_modified_timer = Some(index);
        }
        self.device_indices.take();
        self.devices[index].device.downcast_mut::<T>()
    }

    pub(crate) fn devices(&self) -> &[DeviceMapping] {
        &self.devices
    }

    pub(crate) fn drain_output_each(
        &mut self,
        scratch: &mut Vec<u8>,
        mut consume: impl FnMut(&str, &str, &[u8]) -> Result<(), EmulatorError>,
    ) -> Result<(), EmulatorError> {
        self.touch();
        for mapping in &mut self.devices {
            scratch.clear();
            let Some(kind) = mapping.device.drain_output(scratch) else {
                continue;
            };
            if !scratch.is_empty() {
                consume(kind, &mapping.name, scratch)?;
            }
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn fetch(
        &self,
        address: u64,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        if !matches!(width, 1 | 2 | 4) {
            return Err(EmulatorError::Bus {
                address,
                explanation: "unsupported access width".into(),
            });
        }

        self.memory
            .iter()
            .find(|region| region.executable && region.contains(address, width))
            .ok_or_else(|| EmulatorError::Bus {
                address,
                explanation: "instruction fetch outside executable memory"
                    .into(),
            })?
            .read(address, width)
    }

    fn indices(&self) -> &DeviceIndices {
        self.device_indices
            .get_or_init(|| DeviceIndices::new(&self.devices))
    }

    pub(crate) fn inspect(
        &self,
        address: u64,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        self.memory
            .iter()
            .find(|region| region.contains(address, width))
            .ok_or_else(|| EmulatorError::Bus {
                address,
                explanation: "inspection outside memory".into(),
            })?
            .read(address, width)
    }

    pub(crate) fn install_firmware(
        &mut self,
        address: u64,
        bytes: &[u8],
    ) -> Result<(), EmulatorError> {
        self.install_firmware_segment(address, bytes, bytes.len())
    }

    pub(crate) fn install_firmware_segment(
        &mut self,
        address: u64,
        bytes: &[u8],
        memory_size: usize,
    ) -> Result<(), EmulatorError> {
        self.memory
            .iter_mut()
            .find(|region| region.contains(address, memory_size))
            .ok_or_else(|| {
                EmulatorError::Firmware(format!(
                    "firmware segment 0x{address:08x} ({} bytes) is not fully \
                     mapped",
                    memory_size
                ))
            })?
            .install_firmware_segment(address, bytes, memory_size)
    }

    pub(crate) fn memory(&self) -> &[MemoryRegion] {
        &self.memory
    }

    pub(crate) fn new() -> Self {
        Self::with_address_bits(32)
    }

    pub(crate) fn poll_host_inputs(&mut self) {
        self.touch();
        for mapping in &mut self.devices {
            mapping.device.poll_host_inputs();
        }
    }

    pub(crate) fn read(
        &mut self,
        address: u64,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        if !matches!(width, 1 | 2 | 4) {
            return Err(EmulatorError::Bus {
                address,
                explanation: "unsupported access width".into(),
            });
        }

        if let Some(region) = self
            .memory
            .iter()
            .find(|region| region.contains(address, width))
        {
            return region.read(address, width);
        }

        if let Some(mapping) = self
            .devices
            .iter_mut()
            .find(|mapping| mapping.contains(address, width))
        {
            let before = mapping.device.interrupt_signature();
            let result =
                mapping.device.read((address - mapping.base) as u32, width);
            let changed = mapping.device.interrupt_signature() != before;
            self.devices_dirty |= changed;
            self.host_errors_clean.set(false);
            return result.map_err(|error| device_error(address, error));
        }

        Err(EmulatorError::Bus {
            address,
            explanation: "unmapped read".into(),
        })
    }

    pub(crate) fn read_tagged(
        &self,
        address: u64,
    ) -> Result<(u64, bool), EmulatorError> {
        self.memory
            .iter()
            .find(|region| region.contains(address, 8))
            .ok_or_else(|| EmulatorError::Bus {
                address,
                explanation: "capability load outside memory".into(),
            })?
            .read_tagged(address)
    }

    fn scan_host_errors(&self) -> Result<(), EmulatorError> {
        for &index in &self.indices().host_errors {
            let mapping = &self.devices[index];
            if let Some(error) = mapping.device.host_error() {
                return Err(EmulatorError::HostIo(HostError::other(format!(
                    "device {}: {error}",
                    mapping.name
                ))));
            }
        }
        self.host_errors_clean.set(true);
        Ok(())
    }

    pub(crate) fn set_tag(&mut self, address: u64) {
        if let Some(region) = self
            .memory
            .iter_mut()
            .find(|region| region.contains(address, 1))
        {
            region.set_tag(address);
        }
    }

    pub(crate) fn tag_at(&self, address: u64) -> bool {
        self.memory
            .iter()
            .find_map(|region| region.tag_at(address))
            .unwrap_or(false)
    }

    pub(crate) fn touch(&mut self) {
        self.devices_dirty = true;
        self.host_errors_clean.set(false);
    }

    pub(crate) fn with_address_bits(bits: u32) -> Self {
        Self {
            address_limit: 1 << bits,
            device_indices: OnceCell::new(),
            memory: Vec::new(),
            devices: Vec::new(),
            pending_machine_time: None,
            possibly_modified_timer: None,
            devices_dirty: true,
            interrupt_bits: 0,
            host_errors_clean: std::cell::Cell::new(false),
            host_wait: None,
        }
    }

    pub(crate) fn write(
        &mut self,
        address: u64,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError> {
        self.collect_timer_write();
        if !matches!(width, 1 | 2 | 4) {
            return Err(EmulatorError::Bus {
                address,
                explanation: "unsupported access width".into(),
            });
        }

        if let Some(region) = self
            .memory
            .iter_mut()
            .find(|region| region.contains(address, width))
        {
            return region.write(address, width, value);
        }

        if let Some(mapping) = self
            .devices
            .iter_mut()
            .find(|mapping| mapping.contains(address, width))
        {
            self.host_errors_clean.set(false);
            let offset = (address - mapping.base) as u32;
            let before = mapping.device.interrupt_signature();
            let result = mapping.device.write(offset, width, value);
            let changed = mapping.device.interrupt_signature() != before;
            self.devices_dirty |= changed;
            result.map_err(|error| device_error(address, error))?;
            let written_time = if matches!(
                TimerRegister::find(offset),
                Some(TimerRegister::TimeLow | TimerRegister::TimeHigh)
            ) {
                mapping
                    .device
                    .downcast_mut::<Timer64>()
                    .and_then(Timer64::take_written_time)
            } else {
                None
            };
            if let Some(time) = written_time {
                self.set_machine_time(time);
            }
            return Ok(());
        }

        Err(EmulatorError::Bus {
            address,
            explanation: "unmapped write".into(),
        })
    }

    pub(crate) fn write_tagged(
        &mut self,
        address: u64,
        bits: u64,
        tag: bool,
    ) -> Result<(), EmulatorError> {
        self.memory
            .iter_mut()
            .find(|region| region.contains(address, 8))
            .ok_or_else(|| EmulatorError::Bus {
                address,
                explanation: "capability store outside memory".into(),
            })?
            .write_tagged(address, bits, tag)
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

fn device_error(address: u64, error: EmulatorError) -> EmulatorError {
    match error {
        EmulatorError::Bus { explanation, .. } => EmulatorError::Bus {
            address,
            explanation,
        },
        other => other,
    }
}
