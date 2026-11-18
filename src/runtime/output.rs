// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io::{self, BufWriter, Write};
use std::time::{Duration, Instant};

mod presentation;

use crate::machine::host::websocket::uart::WebSocketUartOutput;

pub(crate) use presentation::OutputPresentation;

pub(crate) struct GuestOutput<W: Write> {
    pub(crate) last_flush: Instant,
    uart_links: Vec<WebSocketUartOutput>,
    uart_outputs: Vec<(String, BufWriter<std::fs::File>)>,
    pub(crate) writer: BufWriter<W>,
}

impl<W: Write> GuestOutput<W> {
    pub(crate) fn diagnostic(
        &mut self,
        text: &str,
        presentation: OutputPresentation,
    ) -> io::Result<()> {
        if !self.uart_outputs.is_empty() {
            self.flush()?;
        }
        if presentation.diagnostics_to_stderr() {
            self.flush()?;
            io::stderr().lock().write_all(text.as_bytes())
        } else {
            self.write(text.as_bytes())
        }
    }

    pub(crate) fn flush(&mut self) -> io::Result<()> {
        let output = self.writer.flush();
        let mut result = output;
        for (_, writer) in &mut self.uart_outputs {
            let flush = writer.flush();
            if result.is_ok() {
                result = flush;
            }
        }
        self.last_flush = Instant::now();
        result
    }

    fn flush_if_due(&mut self) -> io::Result<()> {
        let pending = !self.writer.buffer().is_empty()
            || self
                .uart_outputs
                .iter()
                .any(|(_, writer)| !writer.buffer().is_empty());
        if pending && self.last_flush.elapsed() >= Duration::from_millis(16) {
            self.flush()?;
        }
        Ok(())
    }

    pub(crate) fn new(writer: W) -> Self {
        Self {
            last_flush: Instant::now(),
            uart_links: Vec::new(),
            uart_outputs: Vec::new(),
            writer: BufWriter::with_capacity(8192, writer),
        }
    }

    pub(crate) fn present(
        &mut self,
        kind: &str,
        name: &str,
        bytes: &[u8],
        presentation: OutputPresentation,
    ) -> io::Result<()> {
        if let Some(link) = self
            .uart_links
            .iter()
            .find(|link| kind == "UART" && link.device == name)
        {
            link.write(bytes)?;
        } else if let Some((_, writer)) = self
            .uart_outputs
            .iter_mut()
            .find(|(device, _)| kind == "UART" && device == name)
        {
            writer.write_all(bytes)?;
        } else {
            presentation.present(&mut self.writer, kind, name, bytes)?;
        }
        self.flush_if_due()
    }

    pub(crate) fn with_uart_links(
        mut self,
        links: Vec<WebSocketUartOutput>,
    ) -> Self {
        self.uart_links = links;
        self
    }

    pub(crate) fn with_uart_outputs(
        mut self,
        outputs: Vec<crate::runtime::uart::output_binding::UartOutputBinding>,
    ) -> Self {
        self.uart_outputs = outputs
            .into_iter()
            .map(|output| {
                (output.device, BufWriter::with_capacity(8192, output.file))
            })
            .collect();
        self
    }

    pub(crate) fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)?;

        self.flush_if_due()
    }
}
