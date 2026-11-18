// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::WebSettings;

#[derive(Debug, Default)]
pub struct PeripheralSettings {
    pub ethernet: Vec<String>,
    pub gpio: Vec<String>,
    pub i2c: Vec<String>,
    pub spi: Vec<String>,
    pub uart: Vec<String>,
    pub web: WebSettings,
}
