// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::websocket::endpoint::{Kind, i2c, spi};

#[test]
fn a_kind_is_written_in_lower_case() {
    for (kind, text) in [
        (Kind::Ethernet, "\"ethernet\""),
        (Kind::Gpio, "\"gpio\""),
        (Kind::I2c, "\"i2c\""),
        (Kind::Spi, "\"spi\""),
        (Kind::Uart, "\"uart\""),
    ] {
        assert_eq!(serde_json::to_string(&kind).unwrap(), text);
        assert_eq!(serde_json::from_str::<Kind>(text).unwrap(), kind);
    }
}

#[test]
fn an_i2c_address_and_a_chip_select_are_named_under_their_device() {
    assert_eq!(i2c("i2c", 0x48), "i2c/0x48");
    assert_eq!(i2c("sensors", 0x08), "sensors/0x08");
    assert_eq!(spi("spi", 0), "spi/0");
    assert_eq!(spi("spi1", 3), "spi1/3");
}
