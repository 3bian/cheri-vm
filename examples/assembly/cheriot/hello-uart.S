/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a capability for the 16550 made from the memory root: an address, then */
/* bounds, then a store through it. run it with --trap-policy stop */

.include "common.inc"

.globl _start
_start:
    cspecialr cs0, mtdc
    li a0, UART
    csetaddr cs0, cs0, a0
    csetbounds cs0, cs0, 16
    cllc ca1, greeting
next:
    lbu t1, 0(ca1)
    beqz t1, done
    sw t1, 0(cs0)
    cincoffset ca1, ca1, 1
    j next
done:
    ebreak

greeting:
    .asciz "Hello from CHERIoT\n"

routines
