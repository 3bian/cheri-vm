// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::Device;
use crate::machine::devices::InterruptLine;
use crate::machine::devices::gpio::registers::Register;
use crate::machine::devices::gpio::*;

#[test]
fn gpio_does_not_interrupt_on_externally_changed_output_pins() {
    let mut g = Gpio::new();
    g.write(Register::Direction.offset(), 4, 1).unwrap();
    g.write(Register::InterruptMask.offset(), 4, 3).unwrap();
    g.set_inputs(1);
    assert!(g.interrupt().is_none());
    g.set_inputs(3);
    assert!(g.interrupt().is_some());
    g.write(Register::Output.offset(), 4, 1).unwrap();
    assert_eq!(g.read(Register::Pins.offset(), 4).unwrap(), 3);
    g.write(Register::Pending.offset(), 4, 2).unwrap();
    assert!(g.interrupt().is_none());
}

#[test]
fn rising_input_triggers_masked_interrupt() {
    let mut gpio = Gpio::new();
    gpio.write(Register::InterruptMask.offset(), 4, 1).unwrap();
    gpio.set_inputs(1);
    assert_eq!(gpio.interrupt(), Some(InterruptLine::MachineExternal));
    gpio.write(Register::Pending.offset(), 4, 1).unwrap();
    assert_eq!(gpio.interrupt(), None);
}
