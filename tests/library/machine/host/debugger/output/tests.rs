// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use nix::fcntl::FcntlArg;
use nix::fcntl::OFlag;
use nix::fcntl::fcntl;
use std::collections::VecDeque;
use std::io::Read;
use std::os::fd::OwnedFd;

use crate::machine::host::debugger::output::*;

#[test]
fn a_closed_reader_is_reported_and_not_waited_for() {
    let (reader, writer) = nix::unistd::pipe().unwrap();
    drop(reader);
    let mut output = polled(writer);
    output.enqueue(b"lost", true).unwrap();
    assert!(output.pump().is_err());
}

#[test]
fn a_descriptor_that_shares_its_blocking_state_is_never_waited_on() {
    let (reader, writer) = nix::unistd::pipe().unwrap();
    let mut reader = std::fs::File::from(reader);
    let mut output = polled(writer);
    let mut sent = 0;
    loop {
        output.enqueue(&[b'a'; 4096], true).unwrap();
        let before = output.pending();
        while output.pending() > 0 {
            let pending = output.pending();
            output.pump().unwrap();
            if output.pending() == pending {
                break;
            }
        }
        sent += before - output.pending();
        if output.pending() > 0 {
            break;
        }
    }
    assert!(sent > 0);
    let waiting = output.pending();
    assert!(waiting > 0);
    output.pump().unwrap();
    assert_eq!(output.pending(), waiting, "a full pipe takes nothing");
    let mut received = vec![0; sent];
    reader.read_exact(&mut received).unwrap();
    output.pump().unwrap();
    assert!(output.pending() < waiting);
    assert!(received.iter().all(|byte| *byte == b'a'));
}

fn polled(descriptor: OwnedFd) -> DebugOutput {
    DebugOutput {
        delivery: Delivery::Polled,
        descriptor,
        pending: VecDeque::new(),
    }
}

#[test]
fn the_callers_descriptor_keeps_its_blocking_state() {
    let (_reader, writer) = nix::unistd::pipe().unwrap();
    let output = DebugOutput::new(&writer).unwrap();
    let flags =
        OFlag::from_bits_retain(fcntl(&writer, FcntlArg::F_GETFL).unwrap());
    assert!(!flags.contains(OFlag::O_NONBLOCK));
    assert_ne!(output.delivery, Delivery::Socket);
}
