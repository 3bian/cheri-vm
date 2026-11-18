// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod run;
mod session;

use run::Run;
use session::Session;

use crate::cli::RunArgs;

use crate::error::Result;

use std::collections::{BTreeMap, BTreeSet};

use std::sync::mpsc;

use symbols::Symbols;

use wire::{base64, read_message};

mod inspection;

pub(crate) mod symbols;

mod wire;

const SLICE: u64 = 20_000;
const STEP_BUDGET: u64 = 50_000_000;

pub(super) fn serve(arguments: RunArgs) -> Result<u8> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut input = std::io::stdin().lock();
        while let Some(message) = read_message(&mut input) {
            if sender.send(message).is_err() {
                break;
            }
        }
    });
    let mut session = Session {
        sequence: 0,
        arguments: Some(arguments),
        machine: None,
        symbols: Symbols::default(),
        breakpoints: BTreeSet::new(),
        source_breakpoints: BTreeMap::new(),
        instruction_breakpoints: Vec::new(),
        running: None,
        stopped_at: None,
        scratch: Vec::new(),
        finished: false,
        stop_on_traps: false,
    };
    loop {
        let request = if session.running.is_some() && !session.finished {
            match receiver.try_recv() {
                Ok(request) => Some(request),
                Err(mpsc::TryRecvError::Empty) => {
                    session.advance();
                    continue;
                }
                Err(mpsc::TryRecvError::Disconnected) => None,
            }
        } else {
            receiver.recv().ok()
        };
        let Some(request) = request else {
            return Ok(0);
        };
        if request["type"] != "request" {
            continue;
        }
        if !session.handle(&request) {
            return Ok(0);
        }
    }
}
