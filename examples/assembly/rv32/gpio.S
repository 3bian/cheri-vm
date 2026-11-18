/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* GPIO: directions, outputs, and the levels read back */

.include "common.inc"

.globl _start
_start:
    li s0, GPIO
    li t0, 0x0f
    sw t0, 8(s0)
    lw s1, 8(s0)
    show "direction, pins 0 to 3 as outputs: ", s1
    li t0, 0x35
    sw t0, 4(s0)
    lw s1, 4(s0)
    show "the output register:               ", s1
    lw s1, 0(s0)
    show "the pins: only outputs are driven: ", s1
    ebreak

routines
