/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a capability fault: its cause, the value that names the register at */
/* fault and why, and the return through mepcc. run it with --completion */
/* 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    start
    handle handler
    cspecialr cs0, mtdc
    li a0, 0x80002000
    csetaddr cs0, cs0, a0
    ccleartag ca3, cs0
    print "a load through register 13, which has no tag\n"
faulting:
    lw a4, 0(ca3)
    print "back after the handler\n"
    finish

.balign 4
handler:
    csrr a5, mcause
    show "  mcause:                        ", a5
    csrr a4, mtval
    srli a5, a4, 5
    show "  mtval, the register:           ", a5
    andi a5, a4, 31
    show "  mtval, the reason, 2 is a tag: ", a5
    cspecialr ca1, mepcc
    cllc ca2, faulting
    csub a5, ca1, ca2
    show "  mepcc, bytes from the load:    ", a5
    skip
    mret

routines
