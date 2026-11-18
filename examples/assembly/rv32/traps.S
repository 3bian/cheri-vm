/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* five traps, and the cause and value of each in the handler, which steps */
/* over the instruction and returns. run it with --trap-policy vector and */
/* --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    la s0, word
    print "an instruction that does not exist\n"
    .word 0xffffffff
    print "a load that is not aligned\n"
    lw t1, 1(s0)
    print "a store that is not aligned\n"
    sw t1, 2(s0)
    print "a load from an address nothing answers\n"
    li t2, 0x40000000
    lw t1, 0(t2)
    print "a breakpoint\n"
    ebreak
    print "done\n"
    finish

.balign 4
handler:
    csrr s2, mcause
    csrr s3, mtval
    show "  mcause ", s2
    show "  mtval  ", s3
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0
    mret

.balign 4
word:
    .word 0, 0

routines
