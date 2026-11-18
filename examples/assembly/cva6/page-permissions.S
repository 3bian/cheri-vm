/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the user bit and SUM, an execute-only page and MXR, and a read-only */
/* page. run it with --trap-policy vector and --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    open_memory
    entry root, 1, middle, 0x01
    entry middle, 0, leaves, 0x01
    entry leaves, 0, page, 0xd7
    entry leaves, 1, page, 0xc9
    entry leaves, 2, page, 0xc3
    paging_on root
    to_supervisor supervisor

supervisor:
    li s0, 0x40000000
    print "supervisor mode reads a user page\n"
    ld s1, 0(s0)
    li t0, 1 << 18
    csrs sstatus, t0
    ld s1, 0(s0)
    show "with SUM set: ", s1
    li s0, 0x40001000
    print "a read of an execute-only page\n"
    ld s1, 0(s0)
    li t0, 1 << 19
    csrs sstatus, t0
    ld s1, 0(s0)
    show "with MXR set: ", s1
    li s0, 0x40002000
    print "a store to a read-only page\n"
    sd s1, 0(s0)
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
