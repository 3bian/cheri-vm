/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* wfi, sret and a write to satp, each refused in supervisor mode by a */
/* bit of mstatus: TW, TSR and TVM. run it with --trap-policy vector and */
/* --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    open_memory
    li t0, 0x700000
    csrs mstatus, t0
    to_supervisor supervisor

supervisor:
    print "wfi with TW set\n"
    wfi
    print "sret with TSR set\n"
    sret
    print "a write to satp with TVM set\n"
    csrw satp, zero
    ecall

.balign 4
handler:
    csrr s2, mcause
    li t0, 9
    beq s2, t0, done
    show "  mcause ", s2
    csrr s3, mtval
    show "  mtval  ", s3
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0
    mret
done:
    finish

routines
