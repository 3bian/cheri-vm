/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* what the examples of this folder share: where the devices are, how to */
/* print, and how to end. include it first, and put `routines` last, */
/* after the program, where nothing runs into it. */

/* there are sixteen registers. cgp holds the capability for the UART */
/* from `start` on, ctp is used by `show`, and the routines use ct0 to */
/* ct2, ca0 and cra. */

.option norelax

.equ CLINT, 0x02000000
.equ CONTROLLER, 0x0c000000
.equ UART, 0x10000000
.equ SPI, 0x10001000
.equ I2C, 0x10002000
.equ GPIO, 0x10003000
.equ REVOKER, 0x10004000
.equ ETHERNET, 0x20000000
.equ STORAGE, 0x30000000
.equ TOHOST, 0x80100000

/* start: a capability for the UART, made from the memory root in mtdc: */
/* its address, then bounds that take in its line status register */
.macro start
    cspecialr cgp, mtdc
    li a0, UART
    csetaddr cgp, cgp, a0
    csetbounds cgp, cgp, 32
.endm

/* device register, address, length: a capability for a device, the same way */
.macro device register, address, length
    cspecialr \register, mtdc
    li a0, \address
    csetaddr \register, \register, a0
    csetbounds \register, \register, \length
.endm

/* handle label: make the label the trap vector */
.macro handle label
    cllc ct0, \label
    cspecialw mtcc, ct0
.endm

/* skip: in a handler, step over the four-byte instruction that trapped */
.macro skip
    cspecialr ct0, mepcc
    cincoffset ct0, ct0, 4
    cspecialw mepcc, ct0
.endm

/* print "text" */
.macro print text
    cllc ca0, 7f
    j 8f
7:  .asciz "\text"
    .balign 4
8:  cjal put_string
.endm

/* show "label", register: the label, the register's address in */
/* hexadecimal, a new line */
.macro show label, register
    mv tp, \register
    print "\label"
    mv a0, tp
    cjal put_hex
    cjal put_line
.endm

/* finish: tell the emulator the program is complete, for a run started */
/* with --completion 0x80100000 */
.macro finish
    cspecialr ct0, mtdc
    li t1, TOHOST
    csetaddr ct0, ct0, t1
    li t1, 1
    sw t1, 0(ct0)
    sw zero, 4(ct0)
9:  j 9b
.endm

/* routines: the code that `print` and `show` call */
.macro routines
.balign 4
put_hex:
    li t0, 28
1:  srl t1, a0, t0
    andi t1, t1, 15
    addi t1, t1, '0'
    li t2, '9'
    ble t1, t2, 2f
    addi t1, t1, 'a' - '0' - 10
2:  sw t1, 0(cgp)
    addi t0, t0, -4
    bgez t0, 1b
    cret

put_line:
    li t1, '\n'
    sw t1, 0(cgp)
    cret

put_string:
1:  lbu t1, 0(ca0)
    beqz t1, 2f
    sw t1, 0(cgp)
    cincoffset ca0, ca0, 1
    j 1b
2:  cret
.endm
