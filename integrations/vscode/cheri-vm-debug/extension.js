// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

const vscode = require("vscode");

function activate(context) {
  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory("cheri-vm", {
      createDebugAdapterDescriptor(session) {
        const folder = session.workspaceFolder ? session.workspaceFolder.uri.fsPath : undefined;
        return new vscode.DebugAdapterExecutable(
          session.configuration.cheriVm || "cheri-vm",
          ["debug", "--dap"],
          { cwd: folder }
        );
      },
    })
  );
}

module.exports = { activate, deactivate() {} };
