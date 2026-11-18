/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the compressed capability loads, stores and stack-pointer moves: each */
/* is two bytes, and does what the full form does. run it with */
/* --trap-policy stop */

.include "common.inc"

.globl _start
_start:
    start
    cspecialr csp, mtdc
    li a0, 0x80004100
    csetaddr csp, csp, a0
    csetbounds csp, csp, 0x100
    cmove cs0, csp
    .option rvc
one:
    cincoffset csp, csp, 32
two:
    csc cs0, 8(csp)
three:
    clc ca1, 8(csp)
four:
    cincoffset ca2, csp, 16
five:
    csc cs0, 0(ca2)
six:
    clc ca3, 0(ca2)
seven:
    .option norvc
    li a5, two - one
    show "bytes: the stack pointer moved:         ", a5
    li a5, three - two
    show "bytes: a store through it:              ", a5
    li a5, four - three
    show "bytes: a load through it:               ", a5
    li a5, five - four
    show "bytes: an address made from it:         ", a5
    li a5, six - five
    show "bytes: a store through another:         ", a5
    li a5, seven - six
    show "bytes: a load through another:          ", a5
    cgettag a5, ca1
    show "the capability loaded through it, tag:  ", a5
    cgettag a5, ca3
    show "and through the other, tag:             ", a5
    csub a5, ca2, cs0
    show "the address made, bytes above the base: ", a5
    ebreak

routines
