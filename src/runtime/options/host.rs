// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::runtime::{
    options::ethernet::EthernetOptions, options::gpio::GpioOptions,
    options::i2c::I2cOptions, options::spi::SpiOptions,
    options::uart::UartOptions,
};

#[derive(Default)]
pub(crate) struct HostOptions {
    pub ethernet: Vec<EthernetOptions>,
    pub gpio: Vec<GpioOptions>,
    pub i2c: Vec<I2cOptions>,
    pub spi: Vec<SpiOptions>,
    pub uart: Vec<UartOptions>,
    pub web: crate::runtime::request::WebSettings,
}

impl HostOptions {
    pub(crate) fn count(&self) -> usize {
        self.gpio.len()
            + self.i2c.len()
            + self.uart.len()
            + self.spi.len()
            + self.ethernet.len()
    }

    pub(crate) fn has_hardware(&self) -> bool {
        !self.gpio.is_empty()
            || !self.i2c.is_empty()
            || !self.spi.is_empty()
            || !self.ethernet.is_empty()
    }

    pub(crate) fn has_socket(&self) -> bool {
        self.ethernet
            .iter()
            .any(|option| option.host.socket().is_some())
            || self
                .gpio
                .iter()
                .any(|option| option.host.socket().is_some())
            || self.i2c.iter().any(|option| option.host.socket().is_some())
            || self.spi.iter().any(|option| option.host.socket().is_some())
            || self
                .uart
                .iter()
                .any(|option| option.host.socket().is_some())
    }
}
