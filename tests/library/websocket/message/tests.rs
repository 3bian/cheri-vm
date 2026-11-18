// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::websocket::endpoint::Kind;
use crate::websocket::message::{Message, Reason, ReplyError, Role};

#[test]
fn a_message_is_refused_from_a_role_that_does_not_send_it() {
    for (text, from) in [
        (r#"{"type":"i2c","id":1,"write":"","read":1}"#, Role::Device),
        (r#"{"type":"spi","selected":true}"#, Role::Device),
        (r#"{"type":"sync","id":1}"#, Role::Device),
        (
            r#"{"type":"hello","format_version":1,"profile":"p","device":"uart","kind":"uart","write_only":false}"#,
            Role::Device,
        ),
        (
            r#"{"type":"bye","reason":"finished","message":""}"#,
            Role::Device,
        ),
        (r#"{"type":"reply","id":1,"data":""}"#, Role::Machine),
        (r#"{"type":"ready"}"#, Role::Machine),
    ] {
        assert!(
            refused(text, from).ends_with("is not sent by this role"),
            "{text}"
        );
    }
    parsed(r#"{"type":"error","message":"m"}"#, Role::Machine);
    parsed(r#"{"type":"error","message":"m"}"#, Role::Device);
}

#[test]
fn a_message_is_written_without_the_fields_it_does_not_carry() {
    let hello = Message::Hello {
        format_version: 1,
        profile: "cheriot".into(),
        device: "spi/0".into(),
        kind: Kind::Spi,
        write_only: true,
    };
    assert_eq!(
        hello.text(),
        r#"{"type":"hello","format_version":1,"profile":"cheriot","device":"spi/0","kind":"spi","write_only":true}"#
    );
    let levels = Message::Gpio {
        levels: 0x8000_0001,
        direction: Some(u32::MAX),
        mask: None,
    };
    assert_eq!(
        levels.text(),
        r#"{"type":"gpio","levels":2147483649,"direction":4294967295}"#
    );
    assert_eq!(Message::Ready {}.text(), r#"{"type":"ready"}"#);
    assert_eq!(Message::Sync { id: 4 }.text(), r#"{"type":"sync","id":4}"#);
    let reply = Message::Reply {
        id: 7,
        data: None,
        error: Some(ReplyError::Nack),
        message: None,
    };
    assert_eq!(reply.text(), r#"{"type":"reply","id":7,"error":"nack"}"#);
    let bye = Message::Bye {
        reason: Reason::Finished,
        message: "exit code 0".into(),
    };
    assert_eq!(
        bye.text(),
        r#"{"type":"bye","reason":"finished","message":"exit code 0"}"#
    );
}

#[test]
fn a_message_of_more_than_a_mebibyte_is_refused() {
    let data = "A".repeat(1024 * 1024);
    let text = format!(r#"{{"type":"uart","data":"{data}"}}"#);
    assert_eq!(refused(&text, Role::Device), "a message is at most 1 MiB");
}

#[test]
fn every_message_of_the_reference_page_parses_and_is_written_back_the_same() {
    let page = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../book/appendix-c-messages.md");
    let Ok(page) = std::fs::read_to_string(page) else {
        return;
    };
    let mut found = 0;
    for line in page.lines() {
        let Some((sender, text)) = line
            .strip_prefix("| ")
            .and_then(|row| row.split_once(" | `{"))
            .and_then(|(sender, rest)| {
                Some((sender, rest.strip_suffix("` |")?))
            })
        else {
            continue;
        };
        let text = format!("{{{text}");
        let from = match sender {
            "Machine" => Role::Machine,
            "Device" => Role::Device,
            other => panic!("the messages page has a message from {other}"),
        };
        let message = parsed(&text, from);
        let written: serde_json::Value =
            serde_json::from_str(&message.text()).unwrap();
        let read: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(written, read, "{text}");
        found += 1;
    }
    assert!(found >= 20, "the messages page shows {found} messages");
}

fn parsed(text: &str, from: Role) -> Message {
    Message::parse(text, from).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn refused(text: &str, from: Role) -> String {
    match Message::parse(text, from) {
        Ok(message) => panic!("{text} parsed as {message:?}"),
        Err(error) => error,
    }
}

#[test]
fn unknown_types_and_fields_are_refused_not_ignored() {
    for text in [
        r#"{"type":"wave"}"#,
        r#"{"type":"join","role":"device","token":""}"#,
        r#"{"type":"claim","endpoints":["uart"]}"#,
        r#"{"type":"ready","extra":1}"#,
        r#"{"type":"uart","endpoint":"uart","data":""}"#,
        r#"{"type":"uart","data":"","speed":9600}"#,
        r#"{"type":"reply","id":1,"data":"","more":true}"#,
        r#"{"data":""}"#,
        r#"["uart"]"#,
        "",
    ] {
        refused(text, Role::Device);
    }
    let hello = r#"{"type":"hello","format_version":1,"profile":"p","device":"uart","kind":"uart","write_only":false,"pins":3}"#;
    refused(hello, Role::Machine);
}

#[test]
fn values_out_of_range_are_refused() {
    for (text, from) in [
        (r#"{"type":"uart","data":"not base64!"}"#, Role::Device),
        (r#"{"type":"ethernet","frame":"a"}"#, Role::Device),
        (
            r#"{"type":"gpio","levels":4294967296,"mask":1}"#,
            Role::Device,
        ),
        (r#"{"type":"gpio","levels":-1,"mask":1}"#, Role::Device),
        (r#"{"type":"gpio","levels":1}"#, Role::Device),
        (r#"{"type":"gpio","levels":1,"direction":1}"#, Role::Device),
        (r#"{"type":"gpio","levels":1,"mask":1}"#, Role::Machine),
        (r#"{"type":"spi","mode":4,"data":"AA=="}"#, Role::Machine),
        (r#"{"type":"spi","mode":0}"#, Role::Machine),
        (r#"{"type":"spi","selected":true,"id":1}"#, Role::Machine),
        (
            r#"{"type":"spi","mode":0,"data":"AA==","id":0}"#,
            Role::Machine,
        ),
        (
            r#"{"type":"i2c","id":0,"write":"","read":1}"#,
            Role::Machine,
        ),
        (
            r#"{"type":"i2c","id":1,"write":"","read":4097}"#,
            Role::Machine,
        ),
        (r#"{"type":"sync","id":0}"#, Role::Machine),
        (r#"{"type":"reply","id":1}"#, Role::Device),
        (
            r#"{"type":"reply","id":1,"data":"","error":"nack"}"#,
            Role::Device,
        ),
        (
            r#"{"type":"reply","id":1,"data":"","message":"m"}"#,
            Role::Device,
        ),
        (
            r#"{"type":"bye","reason":"bored","message":""}"#,
            Role::Machine,
        ),
    ] {
        refused(text, from);
    }
}
