// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AesByteOperation {
    Decrypt,
    DecryptMix,
    Encrypt,
    EncryptMix,
}
