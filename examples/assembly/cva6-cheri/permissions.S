/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* permissions: bits are taken away and never given back. a capability */
/* without write faults on a store, and one without read on a load. a */
/* bit that means nothing without another goes with it: store-local with */
/* write, the two load bits with read, system with execute. run it with */
/* --trap-policy vector and --completion 0x80100000 */

.include "common.inc"

.equ WRITE, 1
.equ READ, 1 << 18
.equ EXECUTE, 1 << 17

.globl _start
_start:
    start
    handle handler
    point s0, data
    ybndswi s0, s0, 16
    ypermr s1, s0
    show "all of them:                   ", s1
    li t0, WRITE
    ypermc s2, s0, t0
    ypermr s1, s2
    show "without write:                 ", s1
    ld s1, 0(s2)
    show "a load through it:             ", s1
    print "a store through it\n"
    sd zero, 0(s2)
    li t0, READ
    ypermc s3, s0, t0
    ypermr s1, s3
    show "without read:                  ", s1
    print "a load through that\n"
    ld s1, 0(s3)
    yss s1, s0, s2
    show "the lesser is within the full: ", s1
    yss s1, s2, s0
    show "the full within the lesser:    ", s1
    li t0, EXECUTE
    root s4
    ypermc s4, s4, t0
    ypermr s1, s4
    show "the root without execute:      ", s1
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
    .quad 0x0123456789abcdef
    .quad 0

routines
