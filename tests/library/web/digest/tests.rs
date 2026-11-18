// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::web::digest::{Sha1, hex};

const ONE_MILLION: usize = 1_000_000;

const TWO_BLOCK_MESSAGE: &[u8] =
    b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";

#[test]
fn bytes_are_written_as_lower_case_hexadecimal() {
    assert_eq!(hex(&[0x00, 0x0f, 0xa0, 0xff]), "000fa0ff");
}

#[test]
fn input_fed_in_uneven_pieces_hashes_like_the_whole() {
    let message: Vec<u8> =
        (0..1000u32).map(|value| (value % 251) as u8).collect();
    let mut pieces: Vec<&[u8]> = Vec::new();
    let mut rest = message.as_slice();
    for length in [1, 0, 63, 64, 65, 7, 128, 200] {
        let (piece, tail) = rest.split_at(length);
        pieces.push(piece);
        rest = tail;
    }
    pieces.push(rest);
    assert_eq!(sha1(&pieces), sha1(&[&message]));
}

fn sha1(pieces: &[&[u8]]) -> String {
    let mut hash = Sha1::new();
    for piece in pieces {
        hash.update(piece);
    }
    hex(&hash.finish())
}

#[test]
fn sha1_matches_the_standard_for_its_example_messages() {
    assert_eq!(sha1(&[]), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    assert_eq!(sha1(&[b"abc"]), "a9993e364706816aba3e25717850c26c9cd0d89d");
    assert_eq!(
        sha1(&[TWO_BLOCK_MESSAGE]),
        "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
    );
    assert_eq!(
        sha1(&[&vec![b'a'; ONE_MILLION]]),
        "34aa973cd4c4daa4f61eeb2bdbad27316534016f"
    );
}
