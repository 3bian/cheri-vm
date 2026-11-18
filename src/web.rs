// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(crate) mod channel;
pub mod digest;
pub(crate) mod handshake;
pub(crate) mod head;
pub(crate) mod pages;
pub mod server;

pub use server::{Server, ServerSettings};

pub(crate) const DEVICES: &str = "/ws/";
