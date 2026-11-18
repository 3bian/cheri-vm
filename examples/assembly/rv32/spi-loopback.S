/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* SPI: a chip selected, bytes sent, and the same bytes back from the */
/* loopback peripheral that the profile attaches */

.include "common.inc"

.globl _start
_start:
    li s0, SPI
    sw zero, 12(s0)
    sw zero, 8(s0)
    li t0, 0x95
    sw t0, 0(s0)
    lw s1, 4(s0)
    show "status after a byte:   ", s1
    lw s1, 0(s0)
    show "0x95 out, and back:    ", s1
    li t0, 0x3c
    sw t0, 0(s0)
    lw s1, 0(s0)
    show "0x3c out, and back:    ", s1
    li t0, -1
    sw t0, 8(s0)
    li t0, 3
    sw t0, 8(s0)
    li t0, 0x77
    sw t0, 0(s0)
    lw s1, 4(s0)
    show "status, nothing there: ", s1
    ebreak

routines
