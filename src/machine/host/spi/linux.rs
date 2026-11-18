// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::path::Path;

#[cfg(target_os = "linux")]
use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};
#[cfg(target_os = "linux")]
use std::io;

use crate::machine::EmulatorError;
use crate::machine::devices::spi::SpiPeripheral;

#[cfg(target_os = "linux")]
struct LinuxSpi {
    burst: Vec<u8>,
    device: Spidev,
    mode: Option<u8>,
    selected: bool,
    write_only: bool,
}

#[cfg(target_os = "linux")]
impl LinuxSpi {
    fn flush(&mut self) -> io::Result<()> {
        if self.burst.is_empty() {
            return Ok(());
        }
        let mut transfer = SpidevTransfer::write(&self.burst);
        transfer.cs_change = 1;
        let sent = self.device.transfer(&mut transfer);
        self.burst.clear();
        sent
    }
}

#[cfg(target_os = "linux")]
impl Drop for LinuxSpi {
    fn drop(&mut self) {
        let _ = self.select(false);
    }
}

#[cfg(target_os = "linux")]
impl SpiPeripheral for LinuxSpi {
    fn select(
        &mut self,
        selected: bool,
    ) -> crate::machine::error::HostResult<()> {
        if !selected && self.selected {
            self.flush()?;
            self.device
                .transfer(&mut SpidevTransfer::read_write(&[], &mut []))?;
            self.mode = None;
        }
        self.selected = selected;
        Ok(())
    }

    fn transfer(
        &mut self,
        outgoing: u8,
        mode: u8,
    ) -> crate::machine::error::HostResult<u8> {
        if !self.selected {
            return Err(io::Error::other(
                "SPI transfer requires chip selection",
            )
            .into());
        }
        if self.mode.is_some_and(|previous| previous != mode) {
            return Err(io::Error::other(
                "deselect SPI before changing its mode",
            )
            .into());
        }
        if self.mode.is_none() {
            self.device.configure(
                &SpidevOptions::new()
                    .mode(SpiModeFlags::from_bits_truncate(u32::from(mode)))
                    .build(),
            )?;
            self.mode = Some(mode);
        }
        if self.write_only {
            self.burst.push(outgoing);
            if self.burst.len() == crate::websocket::MAX_BURST_BYTES {
                self.flush()?;
            }
            return Ok(0xff);
        }
        let mut incoming = [0];
        let outgoing = [outgoing];
        let mut transfer = SpidevTransfer::read_write(&outgoing, &mut incoming);
        transfer.cs_change = 1;
        self.device.transfer(&mut transfer)?;
        Ok(incoming[0])
    }
}

pub(crate) fn open(
    path: &Path,
    speed_hz: u32,
    write_only: bool,
) -> Result<Box<dyn SpiPeripheral>, EmulatorError> {
    if !(1..=50_000_000).contains(&speed_hz) {
        return Err(EmulatorError::Configuration(
            "SPI speed must be 1..50000000 Hz".into(),
        ));
    }
    crate::machine::host::device_path::validate(path, "SPI")?;
    #[cfg(target_os = "linux")]
    {
        let file = crate::machine::host::device_path::open(path, "SPI")?;
        let mut device = Spidev::new(file);
        device.configure(
            &SpidevOptions::new()
                .bits_per_word(8)
                .lsb_first(false)
                .max_speed_hz(speed_hz)
                .mode(SpiModeFlags::SPI_MODE_0)
                .build(),
        )?;
        Ok(Box::new(LinuxSpi {
            burst: Vec::new(),
            device,
            mode: None,
            selected: false,
            write_only,
        }))
    }
    #[cfg(not(target_os = "linux"))]
    let _ = write_only;
    #[cfg(not(target_os = "linux"))]
    Err(EmulatorError::Configuration(
        "a host SPI bus is reached through Linux spidev, which this system \
         does not have"
            .into(),
    ))
}
