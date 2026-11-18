/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* auipcc and auicgp, and a fetch past the end of the program counter's */
/* bounds. run it with --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    start
    handle handler
here:
    auipcc ca1, 0
    cllc ca2, _start
    csub a5, ca1, ca2
    show "auipcc gives the address it is at, bytes in: ", a5
    cgetperm a5, ca1
    show "with the program counter's permissions:      ", a5
    cmove cs1, cgp
    cspecialr cgp, mtdc
    li a0, 0x80002000
    csetaddr cgp, cgp, a0
    auicgp ca1, 1
    cmove cgp, cs1
    cgetaddr a5, ca1
    show "auicgp with 1, from a cgp of 0x80002000:     ", a5
    cllc cs0, small
    csetbounds cs0, cs0, 4
    print "a jump to four bytes of code, and a fetch past them\n"
    cjalr cra, cs0
after:
    finish

small:
    nop
    nop

.balign 4
handler:
    csrr a5, mcause
    show "  mcause ", a5
    csrr a5, mtval
    show "  mtval  ", a5
    cllc ct0, after
    cspecialw mepcc, ct0
    mret

routines
