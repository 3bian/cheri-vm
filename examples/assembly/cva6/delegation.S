/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* an environment call from user mode, taken in supervisor mode because */
/* medeleg hands it over. run it with --trap-policy vector and */
/* --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, machine_handler
    csrw mtvec, t0
    open_memory
    li t0, 0x100
    csrw medeleg, t0
    to_supervisor supervisor

supervisor:
    la t0, supervisor_handler
    csrw stvec, t0
    to_user user

user:
    print "user mode makes an environment call\n"
    ecall
    print "back in user mode\n"
    li a7, 1
    ecall

.balign 4
supervisor_handler:
    bnez a7, leave
    csrr s2, scause
    show "the supervisor handler, scause ", s2
    csrr t0, sepc
    addi t0, t0, 4
    csrw sepc, t0
    sret
leave:
    ecall

.balign 4
machine_handler:
    csrr s2, mcause
    show "the machine handler, mcause    ", s2
    finish

routines
