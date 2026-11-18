/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* what the examples of this folder share: the RVY instructions, where the */
/* devices are, how to print, and how to end. include it first, and put */
/* `routines` last, after the program, where nothing runs into it. */

/* every register here is a capability, and a load or store goes through */
/* one. an integer instruction leaves an integer: `mv`, `addi` and `la` */
/* give a result without a tag, so a pointer is moved with `ymv` and */
/* `yaddi`, and made with `point`. gp keeps the UART for the whole program */

.include "rvy.inc"
.option norelax

.equ CLINT, 0x02000000
.equ PLIC, 0x0c000000
.equ UART, 0x10000000
.equ TOHOST, 0x80100000

/* root register: a capability for everything. at reset the program */
/* counter is one, and auipc gives a copy of it at this address */
.macro root register
    auipc \register, 0
.endm

/* point register, label: a capability at the label, with the bounds of */
/* the program counter. t6 is used */
.macro point register, label
    root \register
    la t6, \label
    yaddrw \register, \register, t6
.endm

/* device register, address, length: a capability for the registers of */
/* one device, and nothing beside them. t6 is used */
.macro device register, address, length
    root \register
    li t6, \address
    yaddrw \register, \register, t6
    ybndswi \register, \register, \length
.endm

/* start: gp becomes the UART */
.macro start
    device gp, UART, 32
.endm

/* handle label: traps go to the label */
.macro handle label
    point t5, \label
    csrw mtvec, t5
.endm

/* skip: in a handler, return to the instruction after the one that */
/* trapped. mepc is a capability, so it is moved on as one. t5 is used */
.macro skip
    csrr t5, mepc
    yaddi t5, t5, 4
    csrw mepc, t5
    mret
.endm

/* print "text": the text sits in the code and is stepped over */
.macro print text
    j 8f
7:  .asciz "\text"
    .balign 4
8:  point a0, 7b
    jal put_string
.endm

/* show "label", register: the label, the address in the register as */
/* sixteen hexadecimal digits, a new line */
.macro show label, register
    mv s11, \register
    print "\label"
    mv a0, s11
    jal put_hex
    jal put_line
.endm

/* finish: tell the emulator the program is complete, for a run started */
/* with --completion 0x80100000 */
.macro finish
    device t0, TOHOST, 8
    li t1, 1
    sd t1, 0(t0)
9:  j 9b
.endm

/* routines: the code that `print` and `show` call. a link is a sealed */
/* entry that only a return may use, so none of them calls another */
.macro routines
.balign 4
put_hex:
    li t1, 60
1:  srl t2, a0, t1
    andi t2, t2, 15
    addi t2, t2, '0'
    li t3, '9'
    ble t2, t3, 2f
    addi t2, t2, 'a' - '0' - 10
2:  sw t2, 0(gp)
    addi t1, t1, -4
    bgez t1, 1b
    ret

put_line:
    li t0, '\n'
    sw t0, 0(gp)
    ret

put_string:
1:  lbu t0, 0(a0)
    beqz t0, 2f
    sw t0, 0(gp)
    yaddi a0, a0, 1
    j 1b
2:  ret
.endm
