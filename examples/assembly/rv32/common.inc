/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* what the examples of this folder share: where the devices are, how to */
/* print, and how to end. include it first, and put `routines` last, */
/* after the program, where nothing runs into it. */

.option norelax

.equ CLINT, 0x02000000
.equ CONTROLLER, 0x0c000000
.equ UART, 0x10000000
.equ SPI, 0x10001000
.equ I2C, 0x10002000
.equ GPIO, 0x10003000
.equ ETHERNET, 0x20000000
.equ STORAGE, 0x30000000
.equ TOHOST, 0x80100000

/* print "text": a0 is left pointing at the text, which sits in the code */
.macro print text
    jal a0, 8f
    .asciz "\text"
    .balign 4
8:  call put_string
.endm

/* show "label", register: the label, the register in hexadecimal, a new line */
.macro show label, register
    mv s11, \register
    print "\label"
    mv a0, s11
    call put_hex
    call put_line
.endm

/* finish: tell the emulator the program is complete, for a run started */
/* with --completion 0x80100000 */
.macro finish
    li t0, TOHOST
    li t1, 1
    sw t1, 0(t0)
    sw zero, 4(t0)
9:  j 9b
.endm

/* routines: the code that `print` and `show` call */
.macro routines
.balign 4
put_char:
    li t0, UART
    sw a0, 0(t0)
    ret

put_hex:
    li t0, UART
    li t1, 28
1:  srl t2, a0, t1
    andi t2, t2, 15
    addi t2, t2, '0'
    li t3, '9'
    ble t2, t3, 2f
    addi t2, t2, 'a' - '0' - 10
2:  sw t2, 0(t0)
    addi t1, t1, -4
    bgez t1, 1b
    ret

put_line:
    li t0, UART
    li t1, '\n'
    sw t1, 0(t0)
    ret

put_string:
    li t0, UART
1:  lbu t1, 0(a0)
    beqz t1, 2f
    sw t1, 0(t0)
    addi a0, a0, 1
    j 1b
2:  ret
.endm
