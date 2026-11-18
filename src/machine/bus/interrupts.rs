// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Bus;
use super::{DeviceIndices, EmulatorError, InterruptLine};

impl Bus {
    pub(crate) fn interrupt_pending(&self, line: InterruptLine) -> bool {
        let indices = self.indices();
        let devices = match line {
            InterruptLine::MachineExternal => &indices.external_interrupts,
            InterruptLine::MachineSoftware => &indices.software_interrupts,
            InterruptLine::MachineTimer => &indices.timer_interrupts,
            InterruptLine::SupervisorExternal => &indices.supervisor_interrupts,
        };

        devices
            .iter()
            .any(|&index| self.devices[index].device.interrupt_pending(line))
    }

    pub(super) fn live_interrupt_bits(&self) -> u32 {
        (u32::from(self.interrupt_pending(InterruptLine::MachineExternal))
            << 11)
            | (u32::from(
                self.interrupt_pending(InterruptLine::MachineSoftware),
            ) << 3)
            | (u32::from(self.interrupt_pending(InterruptLine::MachineTimer))
                << 7)
            | (u32::from(
                self.interrupt_pending(InterruptLine::SupervisorExternal),
            ) << 9)
    }

    #[inline]
    pub(crate) fn machine_interrupt_bits(&self) -> u32 {
        if self.devices_dirty {
            self.live_interrupt_bits()
        } else {
            self.interrupt_bits
        }
    }

    pub(super) fn recompute_interrupts(&mut self) {
        use crate::machine::devices::interrupt_controller::InterruptController;
        use crate::machine::devices::plic::Plic;
        let indices = self
            .device_indices
            .get_or_init(|| DeviceIndices::new(&self.devices));
        let mut levels = 0;

        for &(index, source) in &indices.routed_interrupts {
            let device = &mut self.devices[index].device;
            if device.interrupt().is_some() {
                levels |= 1 << source;
                if indices.edge_sources & (1 << source) != 0 {
                    device.interrupt_taken();
                }
            }
        }

        for &index in &indices.interrupt_controllers {
            let device = &mut self.devices[index].device;
            if let Some(controller) =
                device.downcast_mut::<InterruptController>()
            {
                controller.set_levels(levels, indices.edge_sources);
            } else if let Some(controller) = device.downcast_mut::<Plic>() {
                controller.set_levels(levels);
            }
        }
        self.interrupt_bits = self.live_interrupt_bits();
        self.devices_dirty = false;
    }

    #[inline]
    pub(crate) fn refresh_interrupts(&mut self) {
        if self.devices_dirty {
            self.recompute_interrupts();
        }
    }

    pub(crate) fn set_interrupt_source(
        &mut self,
        name: &str,
        source: Option<u8>,
    ) -> Result<(), EmulatorError> {
        if source.is_some_and(|source| source == 0 || source >= 32) {
            return Err(EmulatorError::Configuration(
                "interrupt source must be from 1 to 31".into(),
            ));
        }
        let mapping = self
            .devices
            .iter_mut()
            .find(|mapping| mapping.name == name)
            .ok_or_else(|| {
                EmulatorError::Configuration(format!("no device named {name}"))
            })?;
        self.device_indices.take();
        mapping.interrupt_source = source;
        self.devices_dirty = true;
        Ok(())
    }
}
