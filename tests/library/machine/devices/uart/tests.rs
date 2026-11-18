// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::Device;
use crate::machine::devices::InterruptLine;
use crate::machine::devices::uart::registers::LINE_STATUS_TRANSMIT_EMPTY;
use crate::machine::devices::uart::registers::Ns16550Register;
use crate::machine::devices::uart::registers::Register;
use crate::machine::devices::uart::registers::STATUS_TRANSMIT_READY;
use crate::machine::devices::uart::*;

impl Uart {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn take_transmitted(&mut self) -> Vec<u8> {
        self.transmitted.take()
    }

    pub(crate) fn transmitted(&self) -> &[u8] {
        self.transmitted.as_slice()
    }
}

#[test]
fn a_16550_keeps_its_divisor_behind_the_latch_and_its_scratch_byte() {
    let offset = |register: Ns16550Register| register.offset();
    let mut uart = Uart::with_layout(UartLayout::Ns16550);
    uart.write(offset(Ns16550Register::LineControl), 4, 0x83)
        .unwrap();
    uart.write(offset(Ns16550Register::Data), 4, 0x1b).unwrap();
    uart.write(offset(Ns16550Register::InterruptEnable), 4, 0x02)
        .unwrap();
    assert_eq!(uart.read(offset(Ns16550Register::Data), 4).unwrap(), 0x1b);
    assert_eq!(
        uart.read(offset(Ns16550Register::InterruptEnable), 4)
            .unwrap(),
        2
    );
    uart.write(offset(Ns16550Register::LineControl), 4, 3)
        .unwrap();
    assert_eq!(
        uart.read(offset(Ns16550Register::LineControl), 4).unwrap(),
        3
    );
    assert_eq!(
        uart.read(offset(Ns16550Register::InterruptEnable), 4)
            .unwrap(),
        0
    );
    assert!(uart.transmitted().is_empty());
    uart.write(offset(Ns16550Register::Scratch), 4, 0x5a)
        .unwrap();
    assert_eq!(
        uart.read(offset(Ns16550Register::Scratch), 4).unwrap(),
        0x5a
    );
    assert_eq!(
        uart.read(offset(Ns16550Register::ModemStatus), 4).unwrap(),
        0
    );
}

#[test]
fn a_16550_names_what_interrupts_and_forgets_a_sent_byte_once_it_is_read() {
    let offset = |register: Ns16550Register| register.offset();
    let mut uart = Uart::with_layout(UartLayout::Ns16550);
    assert_eq!(
        uart.read(offset(Ns16550Register::InterruptIdentity), 4)
            .unwrap(),
        1
    );
    assert!(uart.interrupt().is_none());

    uart.write(offset(Ns16550Register::InterruptEnable), 4, 3)
        .unwrap();
    assert!(uart.interrupt().is_some());
    assert_eq!(
        uart.read(offset(Ns16550Register::InterruptIdentity), 4)
            .unwrap(),
        2
    );
    assert!(uart.interrupt().is_none());
    uart.write(offset(Ns16550Register::Data), 4, 65).unwrap();
    assert!(uart.interrupt().is_some());

    uart.inject_receive(b"R").unwrap();
    assert_eq!(
        uart.read(offset(Ns16550Register::InterruptIdentity), 4)
            .unwrap(),
        4
    );
    assert_eq!(uart.read(offset(Ns16550Register::Data), 4).unwrap(), 82);
    assert_eq!(
        uart.read(offset(Ns16550Register::InterruptIdentity), 4)
            .unwrap(),
        2
    );
    assert_eq!(
        uart.read(offset(Ns16550Register::InterruptEnable), 4)
            .unwrap(),
        3
    );
}

#[test]
fn receive_and_transmit_are_independent() {
    let mut uart = Uart::new();
    uart.inject_receive(b"A").unwrap();
    uart.write(Register::Control.offset(), 4, 1).unwrap();
    assert_eq!(uart.interrupt(), Some(InterruptLine::MachineExternal));
    assert_eq!(uart.read(Register::Data.offset(), 4).unwrap(), 65);
    assert_eq!(uart.interrupt(), None);
    uart.write(Register::Data.offset(), 4, 66).unwrap();
    assert_eq!(uart.transmitted(), b"B");
}

#[test]
fn transmitter_readiness_tracks_capture_capacity_for_both_layouts() {
    for (layout, status, ready) in [
        (
            UartLayout::Simple,
            Register::Status.offset(),
            STATUS_TRANSMIT_READY,
        ),
        (
            UartLayout::Ns16550,
            Ns16550Register::LineStatus.offset(),
            LINE_STATUS_TRANSMIT_EMPTY,
        ),
    ] {
        let mut uart = Uart::with_layout(layout);
        uart.inject_receive(b"R").unwrap();
        assert_ne!(uart.read(status, 4).unwrap() & ready, 0);
        for _ in 0..65536 {
            uart.write(Register::Data.offset(), 4, 65).unwrap();
        }
        assert_eq!(uart.read(status, 4).unwrap() & ready, 0, "{layout:?}");
        assert_eq!(uart.read(status, 4).unwrap() & 1, 1);
        assert!(uart.write(Register::Data.offset(), 4, 66).is_err());
        assert_eq!(uart.transmitted().len(), 65536);
        let mut output = Vec::new();
        uart.drain_transmitted(&mut output);
        assert_eq!(output, vec![65; 65536]);
        assert_ne!(uart.read(status, 4).unwrap() & ready, 0);
        uart.write(Register::Data.offset(), 4, 66).unwrap();
        assert_eq!(uart.transmitted(), b"B");
        assert_eq!(
            uart.read(Register::Data.offset(), 4).unwrap(),
            u32::from(b'R')
        );
    }
}

#[test]
fn uart_buffers_are_bounded_and_can_be_drained() {
    let mut uart = Uart::new();
    uart.inject_receive(&vec![42; 4096]).unwrap();
    assert!(uart.inject_receive(b"x").is_err());
    assert_eq!(uart.read(Register::Data.offset(), 4).unwrap(), 42);
    uart.inject_receive(b"x").unwrap();

    for _ in 0..65536 {
        uart.write(Register::Data.offset(), 4, 65).unwrap();
    }

    assert_eq!(
        uart.read(Register::Status.offset(), 4).unwrap()
            & STATUS_TRANSMIT_READY,
        0
    );
    assert!(uart.write(Register::Data.offset(), 4, 66).is_err());
    assert_eq!(uart.take_transmitted().len(), 65536);
    uart.write(Register::Data.offset(), 4, 66).unwrap();
    assert_eq!(uart.transmitted(), b"B");
}
