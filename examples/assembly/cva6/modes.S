/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* machine mode to supervisor mode to user mode, by mret and sret, and */
/* back to machine mode by an environment call. run it with --trap-policy */
/* vector and --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    open_memory
    print "in machine mode\n"
    to_supervisor supervisor

supervisor:
    print "in supervisor mode\n"
    to_user user

user:
    print "in user mode\n"
    ecall

.balign 4
handler:
    csrr s2, mcause
    show "the machine handler, mcause ", s2
    csrr s3, mstatus
    srli s3, s3, 11
    andi s3, s3, 3
    show "the mode it came from       ", s3
    finish

routines
