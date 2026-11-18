/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* sealing and unsealing with the sealing root, and a use while sealed. */
/* a key unseals the types within its bounds, wherever its address is. */
/* run it with --completion 0x80100000 */

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
    cspecialr cs1, mscratchc
    li a0, 9
    csetaddr cs1, cs1, a0
    cseal ca2, cs0, cs1
    cgettype a5, ca2
    show "sealed with type 9, its type:  ", a5
    cgettag a5, ca2
    show "its tag:                       ", a5
    print "a load through it while sealed\n"
    lw a5, 0(ca2)
    cincoffset ca3, ca2, 4
    cgettag a5, ca3
    show "moved while sealed, tag:       ", a5
    cunseal ca3, ca2, cs1
    cgettag a5, ca3
    show "unsealed with type 9, tag:     ", a5
    cgettype a5, ca3
    show "its type:                      ", a5
    li a0, 10
    csetaddr ca4, cs1, a0
    cunseal ca3, ca2, ca4
    cgettag a5, ca3
    show "a key at 10, 9 in its bounds:  ", a5
    csetbounds ca4, ca4, 1
    cunseal ca3, ca2, ca4
    cgettag a5, ca3
    show "a key bounded to 10 alone:     ", a5
    ccleartag ca4, cs1
    cunseal ca3, ca2, ca4
    cgettag a5, ca3
    show "with a key that has no tag:    ", a5
    li a0, ~0x200
    candperm ca4, cs1, a0
    cunseal ca3, ca2, ca4
    cgettag a5, ca3
    show "with a key that cannot unseal: ", a5
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
