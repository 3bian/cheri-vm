// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod gpio_chip;
mod gpio_options;

pub(crate) use gpio_chip::GpioChip;
pub(crate) use gpio_options::GpioOptions;

use crate::machine::EmulatorError;

use super::parse_u32;

fn pins(value: &str) -> Result<u32, EmulatorError> {
    let mut mask = 0_u32;
    for pin in value.split('+') {
        let pin = parse_u32(pin, "GPIO pin")?;
        if pin >= 32 || mask & (1 << pin) != 0 {
            return Err(EmulatorError::Configuration(
                "GPIO pins must be unique and in 0..31".into(),
            ));
        }
        mask |= 1 << pin;
    }
    Ok(mask)
}
