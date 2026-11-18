/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* integers to and from floating point, values out of range, moves, */
/* classify and compare */

.include "common.inc"

.globl _start
_start:
    li t0, 0x2000
    csrs mstatus, t0
    la s0, values
    li t0, -7
    fcvt.d.w fa0, t0
    fmv.x.d s1, fa0
    show "-7 as a double:                 ", s1
    fld fa0, 0(s0)
    fcvt.w.d s1, fa0, rtz
    show "3.7 to a word, toward zero:     ", s1
    fcvt.w.d s1, fa0, rne
    show "3.7 to a word, to nearest:      ", s1
    fld fa1, 8(s0)
    fcvt.w.d s1, fa1, rtz
    show "1e300 to a word is the highest: ", s1
    fld fa2, 16(s0)
    fcvt.wu.d s1, fa2, rtz
    show "-1.0 to an unsigned word is 0:  ", s1
    frflags s1
    show "flags: both were invalid:       ", s1
    fld fa3, 24(s0)
    fcvt.l.d s1, fa3, rtz
    show "not-a-number to a doubleword:   ", s1
    fcvt.s.d fa4, fa0
    fmv.x.w s1, fa4
    show "3.7 narrowed to a single:       ", s1
    fcvt.d.s fa4, fa4
    fmv.x.d s1, fa4
    show "and widened again:              ", s1
    fclass.d s1, fa2
    show "class of -1.0:                  ", s1
    fclass.d s1, fa3
    show "class of a quiet not-a-number:  ", s1
    flt.d s1, fa2, fa0
    show "-1.0 < 3.7:                     ", s1
    feq.d s1, fa3, fa3
    show "not-a-number == itself:         ", s1
    fsgnjn.d fa4, fa0, fa0
    fmv.x.d s1, fa4
    show "3.7 with its sign flipped:      ", s1
    fmin.d fa4, fa0, fa2
    fmv.x.d s1, fa4
    show "the lesser of 3.7 and -1.0:     ", s1
    ebreak

.balign 8
values:
    .double 3.7, 1.0e300, -1.0
    .quad 0x7ff8000000000000

routines
