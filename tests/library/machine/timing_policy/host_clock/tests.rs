// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::hart::machine_registers::MachineRegisters;
use crate::machine::bus::Bus;
use crate::machine::timing_policy::HostClock;

#[test]
fn time_spent_waiting_for_a_host_reply_is_not_guest_time() {
    let waited = Rc::new(Cell::new(Duration::ZERO));
    let mut bus = Bus::new();
    bus.add_host_wait(waited.clone());
    let mut registers = MachineRegisters::<u32>::default();
    let mut clock = HostClock::default();
    clock.set_frequency(1_000_000).unwrap();
    clock.synchronize(&mut bus, &mut registers);
    assert_eq!(registers.time, 0);

    std::thread::sleep(Duration::from_millis(20));
    waited.set(Duration::from_secs(60));
    clock.synchronize(&mut bus, &mut registers);
    assert_eq!(registers.time, 0);
    assert_eq!(waited.get(), Duration::ZERO);

    std::thread::sleep(Duration::from_millis(20));
    waited.set(Duration::from_millis(5));
    clock.synchronize(&mut bus, &mut registers);
    assert!(registers.time >= 15_000, "{}", registers.time);

    let mut plain = Bus::new();
    assert_eq!(plain.take_host_wait(), Duration::ZERO);
}
