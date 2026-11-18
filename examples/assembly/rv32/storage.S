/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* storage: a word of one sector written and read back, and a second */
/* sector left as it was */

.include "common.inc"

.globl _start
_start:
    li s0, STORAGE
    lw s1, 16(s0)
    show "status:                    ", s1
    li t0, 3
    sw t0, 0(s0)
    li t0, 5
    sw t0, 4(s0)
    li t0, 0x12345678
    sw t0, 8(s0)
    li t0, 2
    sw t0, 12(s0)
    lw s1, 16(s0)
    show "status after the write:    ", s1
    sw zero, 8(s0)
    li t0, 5
    sw t0, 4(s0)
    li t0, 1
    sw t0, 12(s0)
    li t0, 5
    sw t0, 4(s0)
    lw s1, 8(s0)
    show "sector 3, word 5, read:    ", s1
    li t0, 4
    sw t0, 0(s0)
    li t0, 1
    sw t0, 12(s0)
    li t0, 5
    sw t0, 4(s0)
    lw s1, 8(s0)
    show "sector 4, word 5, is zero: ", s1
    li t0, 1000
    sw t0, 0(s0)
    li t0, 1
    sw t0, 12(s0)
    lw s1, 16(s0)
    show "status, no such sector:    ", s1
    ebreak

routines
