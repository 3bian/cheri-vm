/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the five rounding modes on one sum, 1 + 2^-53, which lies exactly */
/* between two numbers; and each exception flag raised in turn */

.include "common.inc"

.macro flags label
    frflags s1
    show "\label", s1
    fsflags zero
.endm

.globl _start
_start:
    li t0, 0x2000
    csrs mstatus, t0
    la s0, values
    fld fa0, 0(s0)
    fld fa1, 8(s0)
    fadd.d fa2, fa0, fa1, rne
    fmv.x.d s1, fa2
    show "to nearest, ties to even: ", s1
    fadd.d fa2, fa0, fa1, rtz
    fmv.x.d s1, fa2
    show "toward zero:              ", s1
    fadd.d fa2, fa0, fa1, rdn
    fmv.x.d s1, fa2
    show "down:                     ", s1
    fadd.d fa2, fa0, fa1, rup
    fmv.x.d s1, fa2
    show "up:                       ", s1
    fadd.d fa2, fa0, fa1, rmm
    fmv.x.d s1, fa2
    show "to nearest, ties away:    ", s1
    flags "flags after an inexact sum:  "
    fmv.d.x fa3, zero
    fdiv.d fa2, fa3, fa3
    flags "after 0 / 0, invalid:        "
    fdiv.d fa2, fa0, fa3
    flags "after 1 / 0, divide by zero: "
    fld fa4, 16(s0)
    fmul.d fa2, fa4, fa4
    flags "after an overflow:           "
    fld fa4, 24(s0)
    fmul.d fa2, fa4, fa4
    flags "after an underflow:          "
    ebreak

.balign 8
values:
    .double 1.0
    .quad 0x3ca0000000000000
    .double 1.0e300, 1.0e-300

routines
