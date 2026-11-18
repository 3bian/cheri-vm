/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* supervisor mode with Sv39 on, and one piece of memory mapped twice: by */
/* an entry that lets its pages hold capabilities, bit 58, and by one that */
/* does not. through the second a capability loads without its tag, and */
/* a store of one is a page fault. run it with --trap-policy vector and */
/* --completion 0x80100000 */

.include "common.inc"

.equ CAPABILITIES, 1 << 58
.equ ALIAS, 0x40000000

.globl _start
_start:
    start
    handle handler
    li t0, -1
    csrw pmpaddr0, t0
    li t0, 0x1f
    csrw pmpcfg0, t0
    point s0, table
    li t0, 0xc7
    sd t0, 0(s0)
    li t0, 0x200000c7
    sd t0, 8(s0)
    li t0, 0x200000cf | CAPABILITIES
    sd t0, 16(s0)
    la t0, table
    srli t0, t0, 12
    li t1, 8 << 60
    or t0, t0, t1
    csrw satp, t0
    sfence.vma
    li t0, 0x1800
    csrc mstatus, t0
    li t0, 0x800
    csrs mstatus, t0
    point t5, supervisor
    csrw mepc, t5
    mret

supervisor:
    print "in supervisor mode, with Sv39 on\n"
    point s0, slots
    ybndswi s0, s0, 32
    point s1, value
    ybndswi s1, s1, 8
    sy s1, 0, s0
    ly s2, 0, s0
    ytagr s3, s2
    show "stored and loaded where capabilities may be, tag: ", s3
    root s4
    la t0, slots
    li t1, ALIAS
    sub t0, t0, t1
    yaddrw s4, s4, t0
    ybndswi s4, s4, 32
    ly s2, 0, s4
    ytagr s3, s2
    show "the same memory through the other mapping, tag:   ", s3
    show "its address is there all the same:                ", s2
    mv t0, s1
    sy t0, 16, s4
    print "a capability without a tag is stored through it\n"
    print "one with a tag\n"
    sy s1, 16, s4
    ecall

.balign 4
handler:
    csrr s5, mcause
    li t0, 9
    beq s5, t0, done
    show "  mcause ", s5
    csrr s5, mtval
    show "  mtval  ", s5
    skip
done:
    finish

.balign 16
slots:
    .fill 4, 8, 0
value:
    .quad 0

.balign 4096
table:
    .fill 512, 8, 0

routines
