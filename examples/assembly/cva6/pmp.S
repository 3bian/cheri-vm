/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* PMP: a region locked against machine mode itself, and one kept from */
/* supervisor mode. run it with --trap-policy vector and --completion */
/* 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    la s0, locked
    srli t0, s0, 2
    ori t0, t0, 0x1ff
    csrw pmpaddr0, t0
    la s4, kept
    srli t0, s4, 2
    ori t0, t0, 0x1ff
    csrw pmpaddr1, t0
    li t0, -1
    csrw pmpaddr2, t0
    li t0, 0x1f1899
    csrw pmpcfg0, t0
    ld s1, 0(s0)
    show "machine mode reads the locked region: ", s1
    print "machine mode writes to it\n"
    sd s1, 0(s0)
    to_supervisor supervisor

supervisor:
    print "supervisor mode reads the region kept from it\n"
    ld s1, 0(s4)
    print "supervisor mode reads elsewhere\n"
    la t0, elsewhere
    ld s1, 0(t0)
    show "and gets ", s1
    ecall

.balign 4
handler:
    csrr s2, mcause
    li t0, 9
    beq s2, t0, done
    show "  mcause ", s2
    csrr s3, mtval
    sub s3, s3, s0
    show "  mtval, bytes from the locked region ", s3
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0
    mret
done:
    finish

.balign 8
elsewhere:
    .quad 0x1122334455667788

.balign 4096
locked:
    .quad 0x0123456789abcdef
.balign 4096
kept:
    .quad 0x0fedcba987654321
.balign 4096

routines
