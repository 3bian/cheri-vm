/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* M: the high halves of a product, and division by zero and its overflow */

.include "common.inc"

.globl _start
_start:
    li s0, 0x80000000
    li s1, -1
    li s2, 7
    li s3, -3
    mul s4, s2, s3
    show "mul 7 * -3: ", s4
    mulh s4, s0, s0
    show "mulh, signed, of 0x80000000 squared: ", s4
    mulhu s4, s0, s0
    show "mulhu, unsigned, of the same: ", s4
    mulhsu s4, s1, s0
    show "mulhsu of -1 by unsigned 0x80000000: ", s4
    div s4, s2, s3
    show "div 7 / -3 rounds toward zero: ", s4
    rem s4, s2, s3
    show "rem 7 % -3 takes the sign of 7: ", s4
    div s4, s2, zero
    show "div by zero gives all ones: ", s4
    divu s4, s2, zero
    show "divu by zero gives all ones: ", s4
    rem s4, s2, zero
    show "rem by zero gives the dividend: ", s4
    div s4, s0, s1
    show "div of the lowest number by -1 gives it back: ", s4
    rem s4, s0, s1
    show "rem of the same is zero: ", s4
    ebreak

routines
