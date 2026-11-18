// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{Session, base64};
use serde_json::{Value, json};

impl Session {
    pub(super) fn evaluate(
        &mut self,
        expression: &str,
    ) -> std::result::Result<Value, String> {
        let name = expression.trim().trim_start_matches('$');
        let system = self.system()?;
        let registers = system.registers();
        let words: Vec<&str> = name.split_whitespace().collect();
        let result = if let Some(register) =
            registers.iter().find(|register| register.name == name)
        {
            match register.address {
                Some(address) => self.hex(address),
                None => register.value.clone(),
            }
        } else if let Some(answer) = system.monitor(&words) {
            answer.map_err(str::to_owned)?.trim_end().to_owned()
        } else {
            return Err(format!(
                "cannot evaluate {expression:?}; use pc or a register name \
                 such as {}",
                registers.get(3).map_or("sp", |register| &register.name)
            ));
        };
        Ok(json!({"result": result, "variablesReference": 0}))
    }

    pub(super) fn frames(&mut self) -> Vec<Value> {
        let mut addresses = vec![self.pc()];
        if let Some(link) = self
            .system()
            .ok()
            .and_then(|system| system.return_address())
            && self.symbols.function(link).is_some()
        {
            addresses.push(link);
        }
        addresses
            .iter()
            .enumerate()
            .map(|(index, address)| {
                let reference = self.hex(*address);
                let name = self.symbols.function(*address).map_or_else(
                    || reference.clone(),
                    |(_, _, name)| name.clone(),
                );
                let mut frame = json!({
                    "id": index + 1,
                    "name": name,
                    "line": 0,
                    "column": 0,
                    "instructionPointerReference": reference,
                });
                if let Some((file, line)) = self.symbols.line(*address) {
                    frame["line"] = line.into();
                    frame["source"] = json!({
                        "name":
                            file.file_name().map(|name| name.to_string_lossy()),
                        "path": file.display().to_string(),
                    });
                }
                frame
            })
            .collect()
    }

    pub(super) fn read_memory(
        &mut self,
        arguments: &Value,
    ) -> std::result::Result<Value, String> {
        let reference = arguments["memoryReference"].as_str().unwrap_or("0");
        let base = u64::from_str_radix(reference.trim_start_matches("0x"), 16)
            .map_err(|_| format!("invalid memory reference {reference}"))?;
        let address =
            base.wrapping_add(arguments["offset"].as_i64().unwrap_or(0) as u64);
        let count = arguments["count"].as_u64().unwrap_or(0).min(1024) as usize;
        let mut data = vec![0; count];
        self.system()?
            .inspect(address, &mut data)
            .map_err(|error| error.to_string())?;
        Ok(json!({
            "address": self.hex(address),
            "data": base64(&data),
        }))
    }

    pub(super) fn registers(&mut self) -> Vec<Value> {
        let described = self
            .system()
            .map(|system| system.registers())
            .unwrap_or_default();
        described
            .into_iter()
            .map(|register| {
                let mut variable = json!({
                    "name": register.name,
                    "value": register.value,
                    "variablesReference": 0,
                });
                if let Some(address) = register.address {
                    variable["memoryReference"] = self.hex(address).into();
                }
                variable
            })
            .collect()
    }

    pub(super) fn set_instruction_breakpoints(
        &mut self,
        arguments: &Value,
    ) -> Value {
        let mut addresses = Vec::new();
        let breakpoints: Vec<Value> = arguments["breakpoints"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|breakpoint| {
                let reference =
                    breakpoint["instructionReference"].as_str().unwrap_or("");
                let offset = breakpoint["offset"].as_i64().unwrap_or(0) as u64;
                match u64::from_str_radix(
                    reference.trim_start_matches("0x"),
                    16,
                ) {
                    Ok(address) => {
                        addresses.push(address.wrapping_add(offset));
                        json!({"verified": true})
                    }
                    Err(_) => {
                        json!({"verified": false, "message": "invalid address"})
                    }
                }
            })
            .collect();
        self.instruction_breakpoints = addresses;
        self.rebuild_breakpoints();
        json!({"breakpoints": breakpoints})
    }

    pub(super) fn threads(&mut self) -> Value {
        let name = self.system().map_or("core", |system| system.thread());
        json!({"threads": [{"id": 1, "name": name}]})
    }
}
