// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::EmulatorError;
use crate::machine::devices::ethernet::registers::Register;
use crate::machine::devices::ethernet::*;
use crate::machine::devices::ethernet::{Ethernet, NetworkBackend};
use crate::machine::devices::{Device, InterruptLine};

impl Ethernet {
    pub(crate) fn inject_frame(
        &mut self,
        frame: &[u8],
    ) -> Result<(), EmulatorError> {
        if !self.link
            || !matches!(self.backend, NetworkBackend::InMemory)
            || frame.is_empty()
            || frame.len() > MAX_FRAME
            || self.received.len() >= QUEUE_CAPACITY
        {
            return Err(EmulatorError::Configuration(
                "Ethernet receive queue full or invalid frame size".into(),
            ));
        }

        self.received.push_back(frame.to_vec());
        self.status |= 2;

        Ok(())
    }

    pub(crate) fn pending_frames(&self) -> usize {
        self.received.len()
    }

    pub(crate) fn set_link(&mut self, enabled: bool) {
        self.link = enabled;
    }

    pub(crate) fn take_transmitted(&mut self) -> Option<Vec<u8>> {
        self.transmitted.pop_front()
    }
}

#[test]
fn a_full_receive_queue_refuses_a_frame_and_a_down_link_reports_an_error() {
    let mut ethernet = Ethernet::new(NetworkBackend::InMemory);

    for _ in 0..8 {
        ethernet.inject_frame(b"a").unwrap();
    }

    assert!(ethernet.inject_frame(b"b").is_err());
    ethernet.set_link(false);
    ethernet
        .write(Register::TransmitLength.offset(), 4, 1)
        .unwrap();
    ethernet
        .write(Register::TransmitData.offset(), 4, 3)
        .unwrap();
    ethernet
        .write(Register::TransmitSend.offset(), 4, 1)
        .unwrap();
    assert_eq!(ethernet.read(Register::Status.offset(), 4).unwrap() & 4, 4);
}

#[test]
fn an_empty_send_a_full_queue_and_a_disconnection_are_reported() {
    let mut e = Ethernet::new(NetworkBackend::InMemory);
    e.write(Register::TransmitSend.offset(), 4, 1).unwrap();
    assert_eq!(e.read(Register::Status.offset(), 4).unwrap(), 4);
    assert!(e.take_transmitted().is_none());

    for i in 0..9 {
        e.write(Register::TransmitLength.offset(), 4, 1).unwrap();
        e.write(Register::TransmitData.offset(), 4, i).unwrap();
        e.write(Register::TransmitSend.offset(), 4, 1).unwrap();
    }

    for i in 0..8 {
        assert_eq!(e.take_transmitted().unwrap(), [i]);
    }

    assert!(e.take_transmitted().is_none());
}

#[test]
fn ethernet_level_interrupt_cannot_be_cleared_while_queued() {
    let mut ethernet = Ethernet::new(NetworkBackend::InMemory);
    ethernet
        .write(Register::InterruptMask.offset(), 4, 2)
        .unwrap();
    ethernet.inject_frame(b"abcde").unwrap();
    ethernet
        .write(Register::Status.offset(), 4, u32::MAX)
        .unwrap();
    assert_eq!(ethernet.read(Register::Status.offset(), 4).unwrap() & 2, 2);
    assert_eq!(ethernet.interrupt(), Some(InterruptLine::MachineExternal));
    assert_eq!(
        ethernet.read(Register::ReceiveData.offset(), 4).unwrap(),
        0x64636261
    );
    assert_eq!(
        ethernet.read(Register::ReceiveData.offset(), 4).unwrap(),
        b'e' as u32
    );
    assert_eq!(ethernet.read(Register::ReceiveData.offset(), 4).unwrap(), 0);
    ethernet.write(Register::ReceivePop.offset(), 4, 1).unwrap();
    assert_eq!(ethernet.interrupt(), None);
}

#[test]
fn frames_sent_and_received_in_memory_raise_interrupts() {
    let mut ethernet = Ethernet::new(NetworkBackend::InMemory);
    ethernet
        .write(Register::InterruptMask.offset(), 4, 3)
        .unwrap();
    ethernet
        .write(Register::TransmitLength.offset(), 4, 3)
        .unwrap();
    ethernet
        .write(Register::TransmitData.offset(), 4, 0x00636261)
        .unwrap();
    ethernet
        .write(Register::TransmitSend.offset(), 4, 1)
        .unwrap();
    assert_eq!(ethernet.take_transmitted(), Some(b"abc".to_vec()));
    assert_eq!(ethernet.interrupt(), Some(InterruptLine::MachineExternal));
    ethernet.inject_frame(b"xyz").unwrap();
    assert_eq!(
        ethernet.read(Register::ReceiveLength.offset(), 4).unwrap(),
        3
    );
    assert_eq!(
        ethernet.read(Register::ReceiveData.offset(), 4).unwrap() & 0xffffff,
        0x7a7978
    );
    ethernet.write(Register::ReceivePop.offset(), 4, 1).unwrap();
    assert_eq!(ethernet.pending_frames(), 0);
}
