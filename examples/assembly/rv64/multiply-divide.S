/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* M at 64 bits, and in its word forms */

.include "common.inc"

.globl _start
_start:
    li s0, 0x8000000000000000
    li s1, -1
    li s2, 0x100000003
    li s3, 7
    mul s4, s2, s3
    show "mul  0x100000003 * 7:               ", s4
    mulw s4, s2, s3
    show "mulw sees only the low words:       ", s4
    mulh s4, s0, s0
    show "mulh of the lowest number squared:  ", s4
    mulhu s4, s0, s0
    show "mulhu of the same:                  ", s4
    div s4, s0, s1
    show "div of the lowest number by -1:     ", s4
    li s5, 0x80000000
    divw s4, s5, s1
    show "divw of the lowest word by -1:      ", s4
    divuw s4, s1, s3
    show "divuw 0xffffffff / 7:               ", s4
    remw s4, s2, s3
    show "remw 3 % 7:                         ", s4
    divw s4, s3, zero
    show "divw by zero:                       ", s4
    ebreak

routines
