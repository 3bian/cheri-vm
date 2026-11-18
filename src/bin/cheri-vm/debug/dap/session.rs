// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::symbols::Symbols;
use super::{Run, SLICE, STEP_BUDGET};
use crate::cli::RunArgs;
use cheri_vm::debug::adapter::System;
use cheri_vm::program::output;
use cheri_vm::runtime::AdapterMachine;
use cheri_vm::runtime::request::Firmware;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::PathBuf;

pub(super) struct Session {
    pub(super) sequence: u64,
    pub(super) arguments: Option<RunArgs>,
    pub(super) machine: Option<AdapterMachine>,
    pub(super) symbols: Symbols,
    pub(super) breakpoints: BTreeSet<u64>,
    pub(super) source_breakpoints: BTreeMap<PathBuf, Vec<u64>>,
    pub(super) instruction_breakpoints: Vec<u64>,
    pub(super) running: Option<Run>,
    pub(super) stopped_at: Option<u64>,
    pub(super) scratch: Vec<u8>,
    pub(super) finished: bool,
    pub(super) stop_on_traps: bool,
}

impl Session {
    pub(super) fn advance(&mut self) {
        for _ in 0..SLICE {
            let pc = self.pc();
            let resuming = self.stopped_at.take() == Some(pc);
            if !resuming && self.breakpoints.contains(&pc) {
                return self.stop("breakpoint", None);
            }
            let outcome = match self.system() {
                Ok(system) => system.advance(),
                Err(message) => return self.stop("exception", Some(message)),
            };
            let advanced = match outcome {
                Ok(advanced) => advanced,
                Err(error) => {
                    return self.stop("exception", Some(error.to_string()));
                }
            };
            if let Some(code) = advanced.exit {
                return self.finish(code);
            }
            if let Some(trap) = advanced.trap
                && !trap.interrupt
                && (trap.cause == 3 || self.stop_on_traps)
            {
                let kind = if trap.cause == 3 { "ebreak" } else { "trap" };
                let address = self.hex(trap.address);
                return self.stop(
                    "exception",
                    Some(format!("{kind} at {address} (cause {})", trap.cause)),
                );
            }
            let pc = self.pc();
            let done = match self.running.as_mut() {
                None => true,
                Some(Run::Continue) => false,
                Some(Run::Out { target, budget }) => {
                    *budget = budget.saturating_sub(1);
                    pc == *target || *budget == 0
                }
                Some(Run::Line {
                    start,
                    function,
                    over,
                    budget,
                }) => {
                    *budget = budget.saturating_sub(1);
                    let inside = function
                        .is_none_or(|(low, high)| pc >= low && pc < high);
                    let here = self
                        .symbols
                        .line(pc)
                        .map(|(file, line)| (file.to_path_buf(), line));
                    *budget == 0
                        || (here.is_some()
                            && here != *start
                            && (inside || !*over))
                }
            };
            if done {
                return self.stop("step", None);
            }
        }
        self.drain();
    }

    pub(super) fn drain(&mut self) {
        let Some(machine) = self.machine.as_mut() else {
            return;
        };
        let mut text = String::new();
        machine.system.drain(&mut self.scratch, &mut |bytes| {
            text.push_str(&String::from_utf8_lossy(bytes));
        });
        if !text.is_empty() {
            self.event("output", json!({"category": "stdout", "output": text}));
        }
    }

    pub(super) fn event(&mut self, name: &str, body: Value) {
        let mut event = json!({"type": "event", "event": name});
        event["body"] = body;
        self.send(event);
    }

    pub(super) fn finish(&mut self, code: u64) {
        self.running = None;
        self.finished = true;
        self.drain();
        self.event("exited", json!({"exitCode": code}));
        self.event("terminated", json!({}));
    }

    pub(super) fn handle(&mut self, request: &Value) -> bool {
        let arguments = &request["arguments"];
        let command =
            request["command"].as_str().unwrap_or_default().to_owned();
        let body: std::result::Result<Value, String> = match command.as_str() {
            "initialize" => Ok(json!({
                "supportsConfigurationDoneRequest": true,
                "supportsInstructionBreakpoints": true,
                "supportsSteppingGranularity": true,
                "supportsReadMemoryRequest": true,
                "supportsEvaluateForHovers": true,
                "supportsTerminateRequest": true,
                "exceptionBreakpointFilters": [{
                    "filter": "trap",
                    "label": "Synchronous traps",
                    "description": "Stop when the guest takes a synchronous \
                        trap, before its handler runs",
                    "default": false,
                }],
            })),
            "launch" => self.launch(arguments).map(|()| json!({})),
            "setBreakpoints" => {
                let source = PathBuf::from(
                    arguments["source"]["path"].as_str().unwrap_or(""),
                );
                let mut addresses = Vec::new();
                let breakpoints: Vec<Value> = arguments["breakpoints"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|breakpoint| {
                        let line = breakpoint["line"].as_u64().unwrap_or(0);
                        let found = self.symbols.addresses(&source, line);
                        addresses.extend(&found);
                        match found.first() {
                            Some(address) => json!({
                                "verified": true,
                                "line": line,
                                "instructionReference": self.hex(*address),
                            }),
                            None => json!({
                                "verified": false,
                                "line": line,
                                "message":
                                    "no code was generated for this line",
                            }),
                        }
                    })
                    .collect();
                self.source_breakpoints.insert(source, addresses);
                self.rebuild_breakpoints();
                Ok(json!({"breakpoints": breakpoints}))
            }
            "setInstructionBreakpoints" => {
                Ok(self.set_instruction_breakpoints(arguments))
            }
            "setExceptionBreakpoints" => {
                self.stop_on_traps =
                    arguments["filters"].as_array().is_some_and(|filters| {
                        filters.iter().any(|filter| filter == "trap")
                    });
                Ok(json!({}))
            }
            "threads" => Ok(self.threads()),
            "stackTrace" => {
                let frames = self.frames();
                Ok(json!({"totalFrames": frames.len(), "stackFrames": frames}))
            }
            "scopes" => Ok(json!({"scopes": [{
                "name": "Registers",
                "presentationHint": "registers",
                "variablesReference": 1,
                "expensive": false,
            }]})),
            "variables" => Ok(json!({"variables": self.registers()})),
            "readMemory" => self.read_memory(arguments),
            "evaluate" => {
                self.evaluate(arguments["expression"].as_str().unwrap_or(""))
            }
            "continue" => self.launched(json!({"allThreadsContinued": true})),
            "next" | "stepIn" | "stepOut" => self.launched(json!({})),
            "pause" | "configurationDone" | "disconnect" | "terminate" => {
                Ok(json!({}))
            }
            other => Err(format!("unsupported request: {other}")),
        };
        let succeeded = body.is_ok();
        self.respond(request, body);
        if !succeeded {
            return true;
        }
        match command.as_str() {
            "launch" => self.event("initialized", json!({})),
            "configurationDone" => {
                if self.machine.is_some() {
                    self.stop("entry", None);
                }
            }
            "pause" => {
                if self.running.is_some() {
                    self.stop("pause", None);
                }
            }
            "continue" => self.running = Some(Run::Continue),
            "next" | "stepIn" => {
                let pc = self.pc();
                let instruction = arguments["granularity"] == "instruction";
                let start = self
                    .symbols
                    .line(pc)
                    .map(|(file, line)| (file.to_path_buf(), line));
                self.running = Some(Run::Line {
                    budget: if instruction || start.is_none() {
                        1
                    } else {
                        STEP_BUDGET
                    },
                    start,
                    function: self
                        .symbols
                        .function(pc)
                        .map(|(low, high, _)| (*low, *high)),
                    over: command == "next",
                });
            }
            "stepOut" => {
                let target = self
                    .system()
                    .ok()
                    .and_then(|system| system.return_address())
                    .unwrap_or(0);
                self.running = Some(Run::Out {
                    target,
                    budget: STEP_BUDGET,
                });
            }
            "disconnect" | "terminate" => return false,
            _ => {}
        }
        true
    }

    pub(super) fn hex(&mut self, address: u64) -> String {
        let width = self.system().map_or(8, |system| system.digits()) + 2;
        format!("{address:#0width$x}")
    }

    pub(super) fn launch(
        &mut self,
        arguments: &Value,
    ) -> std::result::Result<(), String> {
        let mut exec = self
            .arguments
            .take()
            .ok_or_else(|| "firmware is already launched".to_owned())?;
        if let Some(program) = arguments["program"].as_str() {
            exec.file = Some(PathBuf::from(program));
            exec.address = None;
        }
        if let Some(profile) = arguments["profile"].as_str() {
            exec.profile = Some(profile.to_owned());
        }
        let request =
            crate::run::request(exec).map_err(|error| error.to_string())?;
        let firmware = match &request.firmware {
            Firmware::Elf(path) | Firmware::Raw { path, .. } => path.clone(),
        };
        let machine = cheri_vm::runtime::adapter_machine(request)
            .map_err(|error| error.to_string())?;
        self.symbols = Symbols::load(&machine.firmware);
        if self.symbols.compressed() {
            let warning = format!(
                "warning: {} has compressed debug sections, which are not \
                 read; line numbers are unavailable\n",
                output::path(&firmware)
            );
            self.event(
                "output",
                json!({"category": "console", "output": warning}),
            );
        }
        self.machine = Some(machine);
        Ok(())
    }

    pub(super) fn launched(
        &mut self,
        body: Value,
    ) -> std::result::Result<Value, String> {
        self.system().map(|_| body)
    }

    pub(super) fn pc(&mut self) -> u64 {
        self.system().map_or(0, |system| system.program_counter())
    }

    pub(super) fn rebuild_breakpoints(&mut self) {
        self.breakpoints = self
            .source_breakpoints
            .values()
            .flatten()
            .chain(&self.instruction_breakpoints)
            .copied()
            .collect();
    }

    pub(super) fn respond(
        &mut self,
        request: &Value,
        body: std::result::Result<Value, String>,
    ) {
        let mut response = json!({
            "type": "response",
            "request_seq": request["seq"],
            "command": request["command"],
            "success": body.is_ok(),
        });
        match body {
            Ok(body) => response["body"] = body,
            Err(message) => response["message"] = message.into(),
        }
        self.send(response);
    }

    pub(super) fn send(&mut self, mut message: Value) {
        self.sequence += 1;
        message["seq"] = self.sequence.into();
        let body = message.to_string();
        let mut out = std::io::stdout().lock();
        let _ = write!(out, "Content-Length: {}\r\n\r\n{body}", body.len());
        let _ = out.flush();
    }

    pub(super) fn stop(&mut self, reason: &str, description: Option<String>) {
        self.running = None;
        self.stopped_at = Some(self.pc());
        self.drain();
        let mut body =
            json!({"reason": reason, "threadId": 1, "allThreadsStopped": true});
        if let Some(description) = description {
            body["description"] = description.clone().into();
            body["text"] = description.into();
        }
        self.event("stopped", body);
    }

    pub(super) fn system(
        &mut self,
    ) -> std::result::Result<&mut dyn System, String> {
        match self.machine.as_mut() {
            Some(machine) => Ok(machine.system.as_mut()),
            None => Err("no firmware has been launched".to_owned()),
        }
    }
}
