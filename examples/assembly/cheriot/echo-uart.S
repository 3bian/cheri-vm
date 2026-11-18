/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* input and output through one bounded capability: each character read */
/* is written back, until a q. run it with --trap-policy stop and */
/* --uart in=/dev/stdin */

.include "common.inc"

.globl _start
_start:
    start
wait:
    lw t0, 0x14(cgp)
    andi t0, t0, 1
    beqz t0, wait
    lw t1, 0(cgp)
    li t2, 'q'
    beq t1, t2, done
    sw t1, 0(cgp)
    j wait
done:
    ebreak

routines
