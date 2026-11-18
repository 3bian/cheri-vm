/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* ld and sd, and lwu beside lw */

.include "common.inc"

.globl _start
_start:
    la s0, doubleword
    li t0, 0x8877665580332211
    sd t0, 0(s0)
    ld s1, 0(s0)
    show "ld:                ", s1
    lw s1, 0(s0)
    show "lw, sign extended: ", s1
    lwu s1, 0(s0)
    show "lwu, zero extended:", s1
    lw s1, 4(s0)
    show "lw of the top half:", s1
    li t0, -1
    sw t0, 4(s0)
    ld s1, 0(s0)
    show "after sw to it:    ", s1
    ebreak

.balign 8
doubleword:
    .quad 0

routines
