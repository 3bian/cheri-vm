/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* permissions taken away with candperm, and the fault from a load, a */
/* store and a jump through a capability without the one each needs. run */
/* it with --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    start
    handle handler
    cspecialr cs0, mtdc
    li a0, 0x80002000
    csetaddr cs0, cs0, a0
    csetbounds cs0, cs0, 64
    cgetperm a5, cs0
    show "permissions to begin with:  ", a5
    li a1, ~0x04
    candperm ca2, cs0, a1
    cgetperm a5, ca2
    show "without store:              ", a5
    print "a store through it\n"
    sw zero, 0(ca2)
    li a1, ~0x20
    candperm ca3, cs0, a1
    cgetperm a5, ca3
    show "without load:               ", a5
    print "a load through it\n"
    lw a5, 0(ca3)
    li a1, 0x04
    candperm ca3, ca2, a1
    cgetperm a5, ca3
    show "a permission cannot be put back: ", a5
    print "a jump through a capability that cannot execute\n"
    cjalr cra, cs0
    finish

.balign 4
handler:
    csrr a5, mcause
    show "  mcause ", a5
    csrr a5, mtval
    show "  mtval  ", a5
    skip
    mret

routines
