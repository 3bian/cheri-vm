// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

"use strict";

const parameters = new URLSearchParams(location.search);

function parameter(name, fallback) {
  return parameters.get(name) || fallback;
}

function decode(text) {
  return Uint8Array.from(atob(text), (symbol) => symbol.charCodeAt(0));
}

function encode(bytes) {
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

function device(name, receive) {
  const status = document.getElementById("status");
  const address = `ws://${location.host}/ws/${name}`;
  const states = (device.states = device.states || new Map());
  let socket = null;
  const show = (state) => {
    states.set(name, state);
    status.textContent = [...states].map(([each, text]) => `${each}: ${text}`).join("; ");
  };
  const send = (message) => {
    if (socket !== null && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(message));
    }
  };
  const connect = () => {
    let ended = "";
    socket = new WebSocket(address);
    socket.onmessage = (event) => {
      const message = JSON.parse(event.data);
      if (message.type === "hello") {
        send({ type: "ready" });
        show(`serving ${message.profile}`);
      } else if (message.type === "sync") {
        send({ type: "reply", id: message.id, data: "" });
      } else if (message.type === "bye") {
        ended = `the run ${message.reason}: ${message.message}`;
      } else if (message.type === "error") {
        ended = `refused: ${message.message}`;
      }
      receive(message);
    };
    socket.onclose = (event) => {
      show(ended || event.reason || "waiting for a run");
      setTimeout(connect, 1000);
    };
  };
  show("connecting");
  connect();
  return send;
}
