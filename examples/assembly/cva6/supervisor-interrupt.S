/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a timer interrupt taken in machine mode and handed to supervisor mode */
/* as its own, the way firmware gives a kernel its timer. run it with */
/* --trap-policy vector and --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, machine_handler
    csrw mtvec, t0
    open_memory
    li t0, 0x20
    csrw mideleg, t0
    li t1, CLINT + 0xbff8
    ld t2, 0(t1)
    addi t2, t2, 1000
    li t1, CLINT + 0x4000
    sd t2, 0(t1)
    li t0, 0x80
    csrs mie, t0
    to_supervisor supervisor

supervisor:
    la t0, supervisor_handler
    csrw stvec, t0
    li t0, 0x20
    csrs sie, t0
    print "supervisor mode waits\n"
    csrsi sstatus, 2
1:  wfi
    beqz s1, 1b
    print "supervisor mode goes on\n"
    ecall

.balign 4
supervisor_handler:
    csrr s2, scause
    show "the supervisor handler, scause ", s2
    li t0, 0x20
    csrc sie, t0
    li s1, 1
    sret

/* the machine handler touches only registers the rest does not use, for */
/* it can interrupt supervisor mode anywhere */
.balign 4
machine_handler:
    csrr s8, mcause
    bgez s8, done
    li s9, CLINT + 0x4000
    li s8, -1
    sd s8, 0(s9)
    li s8, 0x20
    csrs mip, s8
    mret
done:
    finish

routines
