// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::execution::TrapPolicy;

use crate::runtime::output::OutputPresentation;

pub(crate) struct RunOptions {
    pub insights: Option<
        std::rc::Rc<std::cell::RefCell<crate::runtime::insights::Insights>>,
    >,
    pub breakpoint: Vec<u64>,
    pub memory_dump: Option<(u64, usize)>,
    pub presentation: OutputPresentation,
    pub exit_line: crate::runtime::request::ExitLine,
    pub registers:
        Option<crate::runtime::register_selection::RegisterSelection>,
    pub steps: Option<u64>,
    pub trace: bool,
    pub trap_policy: TrapPolicy,
    pub uart: crate::runtime::uart::connections::UartConnections,
    pub web: Option<crate::web::Server>,
    pub websocket: crate::machine::host::websocket::WebSocketConnections,
}
