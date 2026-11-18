// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::web::channel::{Channel, Event, Side};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let near = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (far, _) = listener.accept().unwrap();
    (near, far)
}

fn channels() -> (Channel, Channel) {
    let (near, far) = pair();
    (
        Channel::new(near, Side::Client, Vec::new()).unwrap(),
        Channel::new(far, Side::Server, Vec::new()).unwrap(),
    )
}

fn frame(first: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = vec![first, payload.len() as u8];
    frame.extend_from_slice(payload);
    frame
}

fn refused(bytes: &[u8]) -> (String, Vec<u8>) {
    let (mut near, far) = pair();
    let mut server = Channel::new(far, Side::Client, Vec::new()).unwrap();
    near.write_all(bytes).unwrap();
    let error = server.read().unwrap_err().to_string();
    drop(server);
    let mut answer = Vec::new();
    near.read_to_end(&mut answer).unwrap();
    (error, answer)
}

#[test]
fn a_client_masks_what_it_sends_and_a_server_does_not() {
    let (near, mut far) = pair();
    let client = Channel::new(near, Side::Client, Vec::new()).unwrap();
    client.send("hello").unwrap();
    let mut sent = [0_u8; 11];
    far.read_exact(&mut sent).unwrap();
    assert_eq!(sent[0], 0x81);
    assert_eq!(sent[1], 0x80 | 5);
    let unmasked: Vec<u8> = sent[6..]
        .iter()
        .enumerate()
        .map(|(index, byte)| byte ^ sent[2 + index % 4])
        .collect();
    assert_eq!(unmasked, b"hello");
    let (near, mut far) = pair();
    let server = Channel::new(near, Side::Server, Vec::new()).unwrap();
    server.send("hello").unwrap();
    let mut sent = [0_u8; 7];
    far.read_exact(&mut sent).unwrap();
    assert_eq!(sent, *b"\x81\x05hello");
}

#[test]
fn a_close_is_given_with_its_code_and_reason_and_is_answered() {
    let (client, mut server) = channels();
    client.close(1008, "a rule").unwrap();
    assert_eq!(
        server.read().unwrap(),
        Event::Close {
            code: Some(1008),
            reason: "a rule".into(),
        }
    );
    let mut client = client;
    assert_eq!(
        client.read().unwrap(),
        Event::Close {
            code: Some(1008),
            reason: String::new(),
        }
    );
}

#[test]
fn a_frame_that_breaks_the_protocol_is_refused_with_a_close() {
    for (bytes, reason, code) in [
        (frame(0xc1, b"x"), "reserved bits are set", 1002),
        (
            vec![0x81, 0x81, 1, 2, 3, 4, b'x'],
            "a client masks its frames and a server does not",
            1002,
        ),
        (frame(0x83, b""), "unknown frame", 1002),
        (frame(0x80, b"x"), "fragments out of order", 1002),
        (frame(0x81, &[0xff, 0xfe]), "text is not UTF-8", 1007),
        (
            vec![0x81, 127, 0, 0, 0, 0, 0, 0x10, 0, 1],
            "a message is at most 1 MiB",
            1009,
        ),
        (vec![0x89, 126, 0, 126], "a message is at most 1 MiB", 1009),
    ] {
        let (error, answer) = refused(&bytes);
        assert_eq!(error, reason);
        assert_eq!(answer[0], 0x88, "{reason}");
        let sent = u16::from_be_bytes([answer[6], answer[7]])
            ^ u16::from_be_bytes([answer[2], answer[3]]);
        assert_eq!(sent, code, "{reason}");
    }
}

#[test]
fn a_message_in_fragments_is_put_together_and_a_ping_between_is_answered() {
    let (mut near, far) = pair();
    let mut server = Channel::new(far, Side::Client, Vec::new()).unwrap();
    near.write_all(&frame(0x01, b"one ")).unwrap();
    near.write_all(&frame(0x89, b"beat")).unwrap();
    near.write_all(&frame(0x00, b"two ")).unwrap();
    near.write_all(&frame(0x80, b"three")).unwrap();
    assert_eq!(server.read().unwrap(), Event::Text("one two three".into()));
    let mut pong = [0_u8; 10];
    near.read_exact(&mut pong).unwrap();
    assert_eq!(pong[0], 0x8a);
    assert_eq!(pong[1], 0x80 | 4);
    near.write_all(&frame(0x82, b"\x00")).unwrap();
    assert_eq!(server.read().unwrap(), Event::Binary);
}

#[test]
fn a_read_that_times_out_keeps_what_has_come_for_the_next_read() {
    let (mut near, far) = pair();
    let mut server = Channel::new(far, Side::Client, Vec::new()).unwrap();
    server.stream().set_nonblocking(true).unwrap();
    near.write_all(&[0x81, 5, b'h', b'e']).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    let waiting = server.read().unwrap_err();
    assert_eq!(waiting.kind(), std::io::ErrorKind::WouldBlock);
    near.write_all(b"llo").unwrap();
    server.stream().set_nonblocking(false).unwrap();
    assert_eq!(server.read().unwrap(), Event::Text("hello".into()));
    drop(near);
    let ended = server.read().unwrap_err();
    assert_eq!(ended.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn text_of_every_length_form_crosses_in_both_directions() {
    let (mut client, mut server) = channels();
    for length in [0, 1, 125, 126, 127, 65_535, 65_536, 1024 * 1024] {
        let text =
            "é".repeat(length / 2) + if length % 2 == 1 { "x" } else { "" };
        assert_eq!(text.len(), length);
        let (sender, expected) = (client.writer(), text.clone());
        let sending = std::thread::spawn(move || sender.send(&expected));
        assert_eq!(server.read().unwrap(), Event::Text(text.clone()));
        sending.join().unwrap().unwrap();
        let (sender, expected) = (server.writer(), text.clone());
        let sending = std::thread::spawn(move || sender.send(&expected));
        assert_eq!(client.read().unwrap(), Event::Text(text));
        sending.join().unwrap().unwrap();
    }
}

#[test]
fn what_was_read_with_the_handshake_is_the_start_of_the_first_frame() {
    let (mut near, far) = pair();
    let mut server =
        Channel::new(far, Side::Client, vec![0x81, 3, b'a']).unwrap();
    near.write_all(b"bc").unwrap();
    assert_eq!(server.read().unwrap(), Event::Text("abc".into()));
}
