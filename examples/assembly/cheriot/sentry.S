/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* jumps through the sentries that turn interrupts on and off, and the */
/* return sentry the link makes, which puts them back. run it with */
/* --trap-policy stop */

.include "common.inc"

.macro enabled label
    csrr a5, mstatus
    andi a5, a5, 8
    srli a5, a5, 3
    show "\label", a5
.endm

.globl _start
_start:
    start
    cspecialr cs1, mscratchc
    cllc cs0, routine
    enabled "interrupts enabled, to begin with:       "
    li a0, 3
    csetaddr ca1, cs1, a0
    cseal ca2, cs0, ca1
    cgettype a5, ca2
    show "a sentry sealed with type 3:             ", a5
    cjalr cra, ca2
    show "inside it, interrupts enabled:           ", a4
    show "the type of the link it was given:       ", a3
    enabled "after the return:                        "
    csrsi mstatus, 8
    li a0, 2
    csetaddr ca1, cs1, a0
    cseal ca2, cs0, ca1
    cjalr cra, ca2
    show "through type 2, with them on: inside:    ", a4
    show "the type of the link:                    ", a3
    enabled "after the return:                        "
    ebreak

routine:
    csrr a4, mstatus
    andi a4, a4, 8
    srli a4, a4, 3
    cgettype a3, cra
    cret

routines
