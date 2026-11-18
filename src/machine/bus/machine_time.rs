// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use super::Bus;
use super::{DeviceIndices, MemoryPort, Timer64};

impl Bus {
    pub(crate) fn add_host_wait(&mut self, waited: Rc<Cell<Duration>>) {
        self.host_wait = Some(waited);
    }

    #[inline]
    pub(super) fn collect_timer_write(&mut self) {
        if let Some(index) = self.possibly_modified_timer.take() {
            let written_time = self.devices[index]
                .device
                .downcast_mut::<Timer64>()
                .and_then(Timer64::take_written_time);
            if let Some(time) = written_time {
                self.set_machine_time(time);
            }
        }
    }

    pub(super) fn set_machine_time(&mut self, time: u64) {
        self.touch();
        for mapping in &mut self.devices {
            if let Some(timer) = mapping.device.downcast_mut::<Timer64>() {
                timer.set_time(time);
            }
        }
        self.pending_machine_time = Some(time);
    }

    #[inline]
    pub(crate) fn take_host_wait(&mut self) -> Duration {
        match &self.host_wait {
            Some(waited) => waited.replace(Duration::ZERO),
            None => Duration::ZERO,
        }
    }

    #[inline]
    pub(crate) fn take_machine_time(&mut self) -> Option<u64> {
        if self.possibly_modified_timer.is_none()
            && self.pending_machine_time.is_none()
        {
            return None;
        }
        self.collect_timer_write();
        self.pending_machine_time.take()
    }

    pub(crate) fn tick_devices(&mut self, ticks: u64) {
        self.touch();
        self.collect_timer_write();
        for mapping in &mut self.devices {
            mapping.device.tick(ticks);
        }
        let indices = self
            .device_indices
            .get_or_init(|| DeviceIndices::new(&self.devices));
        for &index in &indices.bus_masters {
            self.devices[index]
                .device
                .drive_memory(ticks, &mut MemoryPort::new(&mut self.memory));
        }
        if let Some(time) = &mut self.pending_machine_time {
            *time = time.wrapping_add(ticks);
        }
    }
}
