/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* Zkn: one round of AES on the state of the standard's own example, and */
/* the four functions of SHA-256 and two of SHA-512 on the first words of */
/* their starting values */

.include "common.inc"

.globl _start
_start:
    li s0, 0x2be2f4a0bee33d19
    li s2, 0x0848f8e92a8dc69a
    aes64esm s1, s0, s2
    show "AES round, low half:   ", s1
    aes64esm s1, s2, s0
    show "AES round, high half:  ", s1
    aes64es s1, s0, s2
    show "the last round's form: ", s1
    aes64ds s1, s0, s2
    show "and its inverse form:  ", s1
    li s0, 0x6a09e667
    sha256sum0 s1, s0
    show "sha256sum0:            ", s1
    sha256sum1 s1, s0
    show "sha256sum1:            ", s1
    sha256sig0 s1, s0
    show "sha256sig0:            ", s1
    sha256sig1 s1, s0
    show "sha256sig1:            ", s1
    li s0, 0x6a09e667f3bcc908
    sha512sum0 s1, s0
    show "sha512sum0:            ", s1
    sha512sig0 s1, s0
    show "sha512sig0:            ", s1
    ebreak

routines
