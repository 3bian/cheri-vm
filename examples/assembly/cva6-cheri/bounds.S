/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* bounds: a capability of 16 bytes, what it reaches, and the fault one */
/* byte further on. then a length the format cannot hold exactly, set */
/* three ways. run it with --trap-policy vector and --completion 0x80100000 */

.include "common.inc"

.globl _start
_start:
    start
    handle handler
    point s0, data
    ybndswi s0, s0, 16
    ybaser s1, s0
    show "base:                               ", s1
    ytopr s1, s0
    show "top:                                ", s1
    ylenr s1, s0
    show "length:                             ", s1
    ld s1, 8(s0)
    show "the last eight bytes inside:        ", s1
    yaddi s2, s0, 16
    ytagr s1, s2
    show "moved to the top, it keeps its tag: ", s1
    print "a load from there\n"
    ld s1, 0(s2)
    print "a store one byte past the end\n"
    sb zero, 16(s0)
    li s3, 0x100001
    point s0, data
    ybndsw s4, s0, s3
    ytagr s1, s4
    show "0x100001 bytes, asked for exactly:  ", s1
    ybndsrw s4, s0, s3
    ytagr s1, s4
    show "the same, rounded outward, tag:     ", s1
    ylenr s1, s4
    show "and the length it was given:        ", s1
    yamask s1, s3
    show "the mask that aligns a base for it: ", s1
    finish

.balign 4
handler:
    csrr s5, mcause
    show "  mcause ", s5
    csrr s5, mtval
    show "  mtval  ", s5
    skip

.balign 16
data:
    .quad 0x1111111111111111
    .quad 0x2222222222222222
    .quad 0x3333333333333333

routines
