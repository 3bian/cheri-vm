/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the four special registers through cspecialrw, and the fault without */
/* the permission to reach them. run it with --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    start
    handle handler
    cspecialr ca1, mtcc
    cllc ca2, _start
    csub a5, ca1, ca2
    show "mtcc, bytes from the start:        ", a5
    cspecialr ca1, mtdc
    cgetperm a5, ca1
    show "mtdc, permissions:                 ", a5
    cspecialr ca1, mscratchc
    cgetperm a5, ca1
    show "mscratchc, permissions:            ", a5
    cspecialr cs0, mtdc
    li a0, 0x80002000
    csetaddr cs0, cs0, a0
    cspecialrw ca1, mscratchc, cs0
    cgetperm a5, ca1
    show "cspecialrw returns what was there: ", a5
    cspecialrw ca1, mscratchc, ca1
    cgetaddr a5, ca1
    show "and put the new value in:          ", a5
    cllc cs0, restricted
    li a0, ~0x80
    candperm cs0, cs0, a0
    print "the same read from code without the permission\n"
    cjalr cra, cs0
after:
    finish

restricted:
    cspecialr ca1, mtdc
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
