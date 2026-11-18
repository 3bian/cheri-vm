// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::Device;
use crate::machine::devices::InterruptLine;
use crate::machine::devices::timer32::registers::Register;
use crate::machine::devices::timer32::*;

#[test]
fn the_timer_wraps_and_counts_large_ticks() {
    let mut t = Timer32::new();
    t.write(Register::Count.offset(), 4, u32::MAX - 1).unwrap();
    t.write(Register::Compare.offset(), 4, 1).unwrap();
    t.write(Register::Control.offset(), 4, 3).unwrap();
    t.tick(2);
    assert_eq!(t.read(Register::Count.offset(), 4).unwrap(), 0);
    assert!(t.interrupt().is_none());
    t.tick(1);
    assert!(t.interrupt().is_some());
    t.write(Register::Pending.offset(), 4, 1).unwrap();
    t.tick((1u64 << 32) - 1);
    assert!(t.interrupt().is_none());
    t.tick(1);
    assert!(t.interrupt().is_some());
    t.tick(u64::MAX);
    assert_eq!(t.read(Register::Count.offset(), 4).unwrap(), 0);
}

#[test]
fn timer_has_no_host_sleep() {
    let mut timer = Timer32::new();
    timer.write(Register::Compare.offset(), 4, 3).unwrap();
    timer.write(Register::Control.offset(), 4, 3).unwrap();
    timer.tick(2);
    assert_eq!(timer.interrupt(), None);
    timer.tick(1);
    assert_eq!(timer.interrupt(), Some(InterruptLine::MachineTimer));
    timer.write(Register::Pending.offset(), 4, 1).unwrap();
    assert_eq!(timer.interrupt(), None);
}
