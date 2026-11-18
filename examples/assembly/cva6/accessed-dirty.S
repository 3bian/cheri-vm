/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the accessed and dirty bits: as on the CVA6, a clear bit is a fault, */
/* and the handler sets it and lets the instruction run again. run it */
/* with --trap-policy vector and --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    open_memory
    entry root, 1, middle, 0x01
    entry middle, 0, leaves, 0x01
    entry leaves, 0, page, 0x07
    paging_on root
    to_supervisor supervisor

supervisor:
    li s0, 0x40000000
    print "a load, with the accessed bit clear\n"
    ld s1, 0(s0)
    show "the load ran again and read ", s1
    print "a store, with the dirty bit clear\n"
    sd s1, 0(s0)
    print "the store ran again\n"
    la t0, leaves
    ld s1, 0(t0)
    andi s1, s1, 0xff
    show "the entry's low bits now ", s1
    ecall

.balign 4
handler:
    csrr s2, mcause
    li t0, 9
    beq s2, t0, done
    show "  mcause ", s2
    la t0, leaves
    ld t1, 0(t0)
    ori t1, t1, 0x40
    li t2, 15
    bne s2, t2, 1f
    ori t1, t1, 0x80
1:  sd t1, 0(t0)
    sfence.vma
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
