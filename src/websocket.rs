// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub mod base64;
pub(crate) mod endpoint;
pub(crate) mod message;

pub(crate) const CLOSE_AWAY: u16 = 1001;
pub(crate) const CLOSE_BINARY: u16 = 1003;
pub(crate) const CLOSE_NORMAL: u16 = 1000;
pub(crate) const CLOSE_PROTOCOL: u16 = 1002;
pub(crate) const CLOSE_RULE: u16 = 1008;
pub(crate) const CLOSE_TOO_LARGE: u16 = 1009;
pub(crate) const CLOSE_UTF8: u16 = 1007;
pub(crate) const FORMAT_VERSION: u32 = 1;
pub(crate) const MAX_BURST_BYTES: usize = 4096;
const MAX_CLOSE_REASON_BYTES: usize = 120;
pub(crate) const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

pub(crate) fn close_reason(reason: &str) -> &str {
    let mut end = reason.len().min(MAX_CLOSE_REASON_BYTES);
    while !reason.is_char_boundary(end) {
        end -= 1;
    }
    &reason[..end]
}
