/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* F and D: arithmetic in single and double precision, and fused */
/* multiply-add. each result is shown as its bits */

.include "common.inc"

.globl _start
_start:
    li t0, 0x2000
    csrs mstatus, t0
    la s0, doubles
    fld fa0, 0(s0)
    fld fa1, 8(s0)
    fadd.d fa2, fa0, fa1
    fmv.x.d s1, fa2
    show "1.5 + 2.25, double:        ", s1
    fmul.d fa2, fa0, fa1
    fmv.x.d s1, fa2
    show "1.5 * 2.25:                ", s1
    fdiv.d fa2, fa1, fa0
    fmv.x.d s1, fa2
    show "2.25 / 1.5:                ", s1
    fsqrt.d fa2, fa1
    fmv.x.d s1, fa2
    show "the square root of 2.25:   ", s1
    fmadd.d fa2, fa0, fa1, fa0
    fmv.x.d s1, fa2
    show "1.5 * 2.25 + 1.5, fused:   ", s1
    fnmsub.d fa2, fa0, fa1, fa0
    fmv.x.d s1, fa2
    show "-(1.5 * 2.25) + 1.5:       ", s1
    la s0, singles
    flw fa0, 0(s0)
    flw fa1, 4(s0)
    fadd.s fa2, fa0, fa1
    fmv.x.w s1, fa2
    show "1.5 + 2.25, single:        ", s1
    fmv.x.d s1, fa2
    show "a single in a wide register:", s1
    fsub.s fa2, fa0, fa1
    fmv.x.w s1, fa2
    show "1.5 - 2.25, single:        ", s1
    ebreak

.balign 8
doubles:
    .double 1.5, 2.25
singles:
    .float 1.5, 2.25

routines
