/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* an environment call taken and returned from, in 64-bit code. run it */
/* with --trap-policy vector and --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    ecall
    print "back from the handler\n"
    finish

.balign 4
handler:
    csrr s2, mcause
    show "mcause ", s2
    csrr s3, mepc
    la t0, _start
    sub s3, s3, t0
    show "mepc, bytes from the start ", s3
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0
    mret

routines
