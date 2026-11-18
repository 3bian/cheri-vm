// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod context;
mod run;

pub use context::Context;
pub use run::Run;

use super::report;

use std::path::Path;

pub fn run<C: Run>(command: C, context: &Context) -> Result<u8, C::Error> {
    let format = report::format(command.format(), command.output(), C::FORMATS);
    let output = command.output().map(Path::to_path_buf);
    let report = command.run(context)?;
    report::emit(&context.shell, report.as_ref(), format, output.as_deref())?;
    Ok(report.code())
}
