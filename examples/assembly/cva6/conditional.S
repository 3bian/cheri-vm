/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* Zicond: a value kept or made zero by a condition, with no branch */

.include "common.inc"

.globl _start
_start:
    li s0, 0x1234
    li s2, 0
    li s3, 7
    czero.eqz s1, s0, s2
    show "czero.eqz, condition zero:    ", s1
    czero.eqz s1, s0, s3
    show "czero.eqz, condition not:     ", s1
    czero.nez s1, s0, s2
    show "czero.nez, condition zero:    ", s1
    czero.nez s1, s0, s3
    show "czero.nez, condition not:     ", s1
    li s4, 0x5678
    czero.eqz t0, s0, s3
    czero.nez t1, s4, s3
    or s1, t0, t1
    show "a select: the first of two:   ", s1
    ebreak

routines
