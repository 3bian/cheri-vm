/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* Sv39: one 4 KiB page mapped at 0x40000000 through three levels of */
/* table, and the fault from the page beside it. run it with */
/* --trap-policy vector and --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    open_memory
    entry root, 1, middle, 0x01
    entry middle, 0, leaves, 0x01
    entry leaves, 0, page, 0xc7
    paging_on root
    to_supervisor supervisor

supervisor:
    li s0, 0x40000000
    ld s1, 0(s0)
    show "read through the mapping: ", s1
    print "the page beside it\n"
    li t0, 0x40001000
    ld s1, 0(t0)
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

.balign 4096
root:
    .fill 512, 8, 0
middle:
    .fill 512, 8, 0
leaves:
    .fill 512, 8, 0
page:
    .quad 0x0123456789abcdef
.balign 4096

routines
