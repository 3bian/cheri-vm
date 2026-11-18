/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a 2 MiB mapping and a 1 GiB mapping: one entry each, higher in the */
/* table. the same memory is read through both. run it with */
/* --trap-policy vector and --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    open_memory
    entry root, 1, middle, 0x01
    entry_at middle, 0, 0x80200000, 0xc7
    entry_at root, 4, 0x80000000, 0xc7
    paging_on root
    to_supervisor supervisor

supervisor:
    li t0, 0x80212340
    li t1, 0x1122334455667788
    sd t1, 0(t0)
    li t0, 0x40012340
    ld s1, 0(t0)
    show "through the 2 MiB mapping: ", s1
    li t0, 0x100212340
    ld s1, 0(t0)
    show "through the 1 GiB mapping: ", s1
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

routines
