// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::arithmetic::{Arithmetic, perform_aes_byte_operation};
use crate::isa::instruction::{
    AesByteOperation, ImmediateOperation, RegisterOperation,
};

const CIPHER: [u64; 2] = [0xfb09_dc02_1d84_2539, 0x320b_6a19_9785_11dc];
const KEY: [u64; 2] = [0xa6d2_ae28_1615_7e2b, 0x3c4f_cf09_8815_f7ab];
const PLAIN: [u64; 2] = [0x8d30_5a88_a8f6_4332, 0x3407_37e0_a298_3131];

fn one(operation: ImmediateOperation, value: u64, immediate: i32) -> u64 {
    value.immediate_operation(operation, immediate)
}

fn primes() -> impl Iterator<Item = u64> {
    (2_u64..).filter(|candidate| {
        (2..*candidate).all(|divisor| candidate % divisor != 0)
    })
}

fn root_fraction(prime: u64, power: u32) -> u64 {
    let mut root = 0_u128;
    for bit in (0..68).rev() {
        let candidate = root | (1 << bit);
        let square = wide_product(candidate, candidate);
        let fits = if power == 2 {
            square <= (u128::from(prime), 0)
        } else {
            let (high, low) = wide_product(candidate, square.1);
            (high + candidate * square.0, low) <= (u128::from(prime) << 64, 0)
        };
        if fits {
            root = candidate;
        }
    }
    root as u64
}

fn round_keys() -> Vec<[u64; 2]> {
    let mut keys = vec![KEY];
    for round in 0..10 {
        let [low, high] = keys[round];
        let scheduled =
            one(ImmediateOperation::AesKeySchedule1, high, round as i32);
        let next_low = two(RegisterOperation::AesKeySchedule2, scheduled, low);
        let next_high = two(RegisterOperation::AesKeySchedule2, next_low, high);
        keys.push([next_low, next_high]);
    }
    keys
}

#[test]
fn the_32_bit_instructions_encrypt_and_decrypt_the_same_example() {
    use AesByteOperation::*;
    let words = |pair: [u64; 2]| {
        [
            pair[0] as u32,
            (pair[0] >> 32) as u32,
            pair[1] as u32,
            (pair[1] >> 32) as u32,
        ]
    };
    let keys: Vec<[u32; 4]> = round_keys().into_iter().map(words).collect();
    let round =
        |operation, state: [u32; 4], key: [u32; 4], step: usize| -> [u32; 4] {
            core::array::from_fn(|column| {
                (0..4).fold(key[column], |sum, row| {
                    perform_aes_byte_operation(
                        operation,
                        sum,
                        state[(column + row * step) % 4],
                        row as u8,
                    )
                })
            })
        };
    let add = |state: [u32; 4], key: [u32; 4]| -> [u32; 4] {
        core::array::from_fn(|column| state[column] ^ key[column])
    };
    let mut state = add(words(PLAIN), keys[0]);
    for key in &keys[1..10] {
        state = round(EncryptMix, state, *key, 1);
    }
    state = round(Encrypt, state, keys[10], 1);
    assert_eq!(state, words(CIPHER));

    state = add(state, keys[10]);
    for key in keys[1..10].iter().rev() {
        let mixed = key.map(|word| {
            (0..4).fold(0, |sum, row| {
                let substituted =
                    perform_aes_byte_operation(Encrypt, 0, word, row);
                perform_aes_byte_operation(DecryptMix, sum, substituted, row)
            })
        });
        state = round(DecryptMix, state, mixed, 3);
    }
    state = round(Decrypt, state, keys[0], 3);
    assert_eq!(state, words(PLAIN));
}

#[test]
fn the_64_bit_instructions_encrypt_and_decrypt_the_standard_example() {
    use RegisterOperation::*;
    let keys = round_keys();
    assert_eq!(keys[10], [0x8925_eec9_a8f9_14d0, 0xa60c_63b6_c80c_3fe1]);
    let mut state = [PLAIN[0] ^ keys[0][0], PLAIN[1] ^ keys[0][1]];
    for key in &keys[1..10] {
        state = [
            two(AesEncryptMix, state[0], state[1]) ^ key[0],
            two(AesEncryptMix, state[1], state[0]) ^ key[1],
        ];
    }
    state = [
        two(AesEncrypt, state[0], state[1]) ^ keys[10][0],
        two(AesEncrypt, state[1], state[0]) ^ keys[10][1],
    ];
    assert_eq!(state, CIPHER);

    state = [state[0] ^ keys[10][0], state[1] ^ keys[10][1]];
    for key in keys[1..10].iter().rev() {
        let mixed =
            key.map(|half| one(ImmediateOperation::AesInverseMix, half, 0));
        state = [
            two(AesDecryptMix, state[0], state[1]) ^ mixed[0],
            two(AesDecryptMix, state[1], state[0]) ^ mixed[1],
        ];
    }
    state = [
        two(AesDecrypt, state[0], state[1]) ^ keys[0][0],
        two(AesDecrypt, state[1], state[0]) ^ keys[0][1],
    ];
    assert_eq!(state, PLAIN);
}

#[test]
fn the_sha_256_instructions_hash_abc() {
    use ImmediateOperation::*;
    let hash = |operation, value: u32| value.immediate_operation(operation, 0);
    let constants: Vec<u32> = primes()
        .take(64)
        .map(|prime| (root_fraction(prime, 3) >> 32) as u32)
        .collect();
    let mut state: Vec<u32> = primes()
        .take(8)
        .map(|prime| (root_fraction(prime, 2) >> 32) as u32)
        .collect();
    let mut schedule = vec![0x6162_6380_u32];
    schedule.extend([0; 14]);
    schedule.push(24);
    for index in 16..64 {
        schedule.push(
            hash(Sha256Sigma1, schedule[index - 2])
                .wrapping_add(schedule[index - 7])
                .wrapping_add(hash(Sha256Sigma0, schedule[index - 15]))
                .wrapping_add(schedule[index - 16]),
        );
    }
    let initial = state.clone();
    for index in 0..64 {
        let [a, b, c, d, e, f, g, h] = state[..] else {
            unreachable!()
        };
        let first = h
            .wrapping_add(hash(Sha256Sum1, e))
            .wrapping_add((e & f) ^ (!e & g))
            .wrapping_add(constants[index])
            .wrapping_add(schedule[index]);
        let second =
            hash(Sha256Sum0, a).wrapping_add((a & b) ^ (a & c) ^ (b & c));
        state = vec![
            first.wrapping_add(second),
            a,
            b,
            c,
            d.wrapping_add(first),
            e,
            f,
            g,
        ];
    }
    let digest: Vec<u32> = state
        .iter()
        .zip(&initial)
        .map(|(word, start)| word.wrapping_add(*start))
        .collect();
    assert_eq!(
        digest,
        [
            0xba78_16bf,
            0x8f01_cfea,
            0x4141_40de,
            0x5dae_2223,
            0xb003_61a3,
            0x9617_7a9c,
            0xb410_ff61,
            0xf200_15ad
        ]
    );
    assert_eq!(
        0xffff_ffff_0000_0001_u64.immediate_operation(Sha256Sum0, 0),
        0x4008_0400
    );
    assert_eq!(
        0x8000_0000_u64.immediate_operation(Sha256Sigma0, 0),
        0x1100_2000
    );
    assert_eq!(
        2_u64.immediate_operation(Sha256Sum0, 0),
        0xffff_ffff_8010_0800
    );
}

#[test]
fn the_sha_512_instructions_hash_abc_and_the_32_bit_halves_agree() {
    use ImmediateOperation::*;
    use RegisterOperation::*;
    let hash = |operation, value: u64| value.immediate_operation(operation, 0);
    let constants: Vec<u64> = primes()
        .take(80)
        .map(|prime| root_fraction(prime, 3))
        .collect();
    let mut state: Vec<u64> = primes()
        .take(8)
        .map(|prime| root_fraction(prime, 2))
        .collect();
    let mut schedule = vec![0x6162_6380_0000_0000_u64];
    schedule.extend([0; 14]);
    schedule.push(24);
    for index in 16..80 {
        schedule.push(
            hash(Sha512Sigma1, schedule[index - 2])
                .wrapping_add(schedule[index - 7])
                .wrapping_add(hash(Sha512Sigma0, schedule[index - 15]))
                .wrapping_add(schedule[index - 16]),
        );
    }
    let initial = state.clone();
    for index in 0..80 {
        let [a, b, c, d, e, f, g, h] = state[..] else {
            unreachable!()
        };
        let first = h
            .wrapping_add(hash(Sha512Sum1, e))
            .wrapping_add((e & f) ^ (!e & g))
            .wrapping_add(constants[index])
            .wrapping_add(schedule[index]);
        let second =
            hash(Sha512Sum0, a).wrapping_add((a & b) ^ (a & c) ^ (b & c));
        state = vec![
            first.wrapping_add(second),
            a,
            b,
            c,
            d.wrapping_add(first),
            e,
            f,
            g,
        ];
    }
    let digest: Vec<u64> = state
        .iter()
        .zip(&initial)
        .map(|(word, start)| word.wrapping_add(*start))
        .collect();
    assert_eq!(
        digest,
        [
            0xddaf_35a1_9361_7aba,
            0xcc41_7349_ae20_4131,
            0x12e6_fa4e_89a9_7ea2,
            0x0a9e_eee6_4b55_d39a,
            0x2192_992a_274f_c1a8,
            0x36ba_3c23_a3fe_ebbd,
            0x454d_4423_643c_e80e,
            0x2a9a_c94f_a54c_a49f
        ]
    );

    for value in schedule.iter().chain(&constants) {
        let (high, low) = ((value >> 32) as u32, *value as u32);
        let joined = |upper: u32, lower: u32| {
            (u64::from(upper) << 32) | u64::from(lower)
        };
        assert_eq!(
            joined(
                high.register_operation(Sha512Sigma0High, low),
                low.register_operation(Sha512Sigma0Low, high)
            ),
            hash(Sha512Sigma0, *value)
        );
        assert_eq!(
            joined(
                high.register_operation(Sha512Sigma1High, low),
                low.register_operation(Sha512Sigma1Low, high)
            ),
            hash(Sha512Sigma1, *value)
        );
        assert_eq!(
            joined(
                high.register_operation(Sha512Sum0Rotated, low),
                low.register_operation(Sha512Sum0Rotated, high)
            ),
            hash(Sha512Sum0, *value)
        );
        assert_eq!(
            joined(
                high.register_operation(Sha512Sum1Rotated, low),
                low.register_operation(Sha512Sum1Rotated, high)
            ),
            hash(Sha512Sum1, *value)
        );
    }
}

fn two(operation: RegisterOperation, first: u64, second: u64) -> u64 {
    first.register_operation(operation, second)
}

fn wide_product(first: u128, second: u128) -> (u128, u128) {
    let mask = u128::from(u64::MAX);
    let (first_high, first_low) = (first >> 64, first & mask);
    let (second_high, second_low) = (second >> 64, second & mask);
    let low = first_low * second_low;
    let middle = first_high * second_low + (low >> 64);
    let other = first_low * second_high + (middle & mask);
    (
        first_high * second_high + (middle >> 64) + (other >> 64),
        (other << 64) | (low & mask),
    )
}
