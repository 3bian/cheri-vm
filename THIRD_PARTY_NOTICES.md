# Third-party notices

`cheri-vm` is licensed under the MIT license in `LICENSE`. This file says what
in the repository comes from somewhere else, whose it is and under which
license.

## The program's own sources

The emulator's sources are derived from `3bian/cheri-vm`, and the code in
`src/program/` from `3bian/cheriot-compose`. Both are MIT licensed,
copyright 3bian Limited and Contributors; the notice is `LICENSE`.

## Test data

None of this is compiled into the program.

- `tests/fixtures/cheriot/hello_world.elf` is a CHERIoT RTOS
  example built for the emulator's board. Its MIT license, copyright Microsoft
  Corporation, is `hello_world.LICENSE` beside it.
- The instruction and control-register vectors in
  `tests/fixtures/vectors/` were generated from the CHERIoT
  Sail model and the RISC-V Sail model, both under the BSD two-clause license.
  Their licenses, with the copyright holders' names, are
  `LICENSE-CHERIOT-REFERENCE.txt` and `LICENSE-RISCV-REFERENCE.txt` in that
  directory.

## Rust packages

The program is built from these packages, which Cargo downloads from
crates.io at the versions `Cargo.lock` records; none of their sources is in
this repository. The table lists every package the lock file names, for
every system, so it holds some that a given build leaves out. Where a license
offers a choice, the program takes the MIT or Apache-2.0 one.

| Package | License |
| --- | --- |
| `async-channel` | Apache-2.0 OR MIT |
| `async-task` | Apache-2.0 OR MIT |
| `atomic-waker` | Apache-2.0 OR MIT |
| `autocfg` | Apache-2.0 OR MIT |
| `bitflags` | MIT OR Apache-2.0 / MIT/Apache-2.0 |
| `block-buffer` | MIT OR Apache-2.0 |
| `blocking` | Apache-2.0 OR MIT |
| `byteorder` | Unlicense OR MIT |
| `bytes` | MIT |
| `cfg-if` | MIT OR Apache-2.0 |
| `cfg_aliases` | MIT |
| `chacha20` | MIT OR Apache-2.0 |
| `concurrent-queue` | Apache-2.0 OR MIT |
| `const-oid` | Apache-2.0 OR MIT |
| `cpufeatures` | MIT OR Apache-2.0 |
| `crossbeam-utils` | MIT OR Apache-2.0 |
| `crypto-common` | MIT OR Apache-2.0 |
| `data-encoding` | MIT |
| `digest` | MIT OR Apache-2.0 |
| `equivalent` | Apache-2.0 OR MIT |
| `event-listener` | Apache-2.0 OR MIT |
| `event-listener-strategy` | Apache-2.0 OR MIT |
| `fallible-iterator` | MIT/Apache-2.0 |
| `fastrand` | Apache-2.0 OR MIT |
| `foldhash` | Zlib |
| `futures` | MIT OR Apache-2.0 |
| `futures-channel` | MIT OR Apache-2.0 |
| `futures-core` | MIT OR Apache-2.0 |
| `futures-executor` | MIT OR Apache-2.0 |
| `futures-io` | MIT OR Apache-2.0 |
| `futures-lite` | Apache-2.0 OR MIT |
| `futures-macro` | MIT OR Apache-2.0 |
| `futures-sink` | MIT OR Apache-2.0 |
| `futures-task` | MIT OR Apache-2.0 |
| `futures-util` | MIT OR Apache-2.0 |
| `gdbstub` | MIT OR Apache-2.0 |
| `getrandom` | MIT OR Apache-2.0 |
| `gimli` | MIT OR Apache-2.0 |
| `gpiocdev` | Apache-2.0 OR MIT |
| `gpiocdev-uapi` | Apache-2.0 OR MIT |
| `hashbrown` | MIT OR Apache-2.0 |
| `http` | MIT OR Apache-2.0 |
| `httparse` | MIT OR Apache-2.0 |
| `hybrid-array` | MIT OR Apache-2.0 |
| `i2cdev` | MIT/Apache-2.0 |
| `indexmap` | Apache-2.0 OR MIT |
| `ioctl-sys` | MIT OR Apache-2.0 |
| `ipnet` | MIT OR Apache-2.0 |
| `itoa` | MIT OR Apache-2.0 |
| `libc` | MIT OR Apache-2.0 |
| `libloading` | ISC |
| `log` | MIT OR Apache-2.0 |
| `managed` | 0BSD |
| `memchr` | Unlicense OR MIT |
| `memoffset` | MIT |
| `nix` | MIT |
| `num-traits` | MIT OR Apache-2.0 |
| `parking` | Apache-2.0 OR MIT |
| `pastey` | MIT OR Apache-2.0 |
| `pin-project-lite` | Apache-2.0 OR MIT |
| `piper` | MIT OR Apache-2.0 |
| `proc-macro2` | MIT OR Apache-2.0 |
| `quote` | MIT OR Apache-2.0 |
| `r-efi` | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| `rand` | MIT OR Apache-2.0 |
| `rand_core` | MIT OR Apache-2.0 |
| `serde` | MIT OR Apache-2.0 |
| `serde_core` | MIT OR Apache-2.0 |
| `serde_derive` | MIT OR Apache-2.0 |
| `serde_json` | MIT OR Apache-2.0 |
| `serde_spanned` | MIT OR Apache-2.0 |
| `sha1` | MIT OR Apache-2.0 |
| `slab` | MIT |
| `spidev` | MIT/Apache-2.0 |
| `stable_deref_trait` | MIT OR Apache-2.0 |
| `syn` | MIT OR Apache-2.0 |
| `thiserror` | MIT OR Apache-2.0 |
| `thiserror-impl` | MIT OR Apache-2.0 |
| `toml` | MIT OR Apache-2.0 |
| `toml_datetime` | MIT OR Apache-2.0 |
| `toml_parser` | MIT OR Apache-2.0 |
| `toml_writer` | MIT OR Apache-2.0 |
| `tun` | WTFPL |
| `tungstenite` | MIT OR Apache-2.0 |
| `typenum` | MIT OR Apache-2.0 |
| `unicode-ident` | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `windows-link` | MIT OR Apache-2.0 |
| `windows-sys` | MIT OR Apache-2.0 |
| `winnow` | MIT |
| `wintun-bindings` | MIT |
| `zmij` | MIT |

Every license here permits use in MIT-licensed programs. One is unusual:
`tun`, which `cheri-vm` uses on Linux for `--ethernet`, is under the WTFPL,
which some organizations do not accept.
