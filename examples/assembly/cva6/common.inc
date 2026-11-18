/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* what the examples of this folder share: where the devices are, how to */
/* print, and how to end. include it first, and put `routines` last, */
/* after the program, where nothing runs into it. */

.option norelax

.equ CLINT, 0x02000000
.equ PLIC, 0x0c000000
.equ UART, 0x10000000
.equ TOHOST, 0x80100000

/* print "text": a0 is left pointing at the text, which sits in the code */
.macro print text
    jal a0, 8f
    .asciz "\text"
    .balign 4
8:  call put_string
.endm

/* show "label", register: the label, the register as sixteen hexadecimal */
/* digits, a new line */
.macro show label, register
    mv s11, \register
    print "\label"
    mv a0, s11
    call put_hex
    call put_line
.endm

/* finish: tell the emulator the program is complete, for a run started */
/* with --completion 0x80100000 */
/* open_memory: one PMP entry that lets supervisor and user mode reach all */
/* of memory, which they cannot until an entry allows it */
.macro open_memory
    li t0, -1
    csrw pmpaddr0, t0
    li t0, 0x1f
    csrw pmpcfg0, t0
.endm

/* to_supervisor label: leave machine mode for supervisor mode at the label */
.macro to_supervisor label
    li t0, 0x1800
    csrc mstatus, t0
    li t0, 0x800
    csrs mstatus, t0
    la t0, \label
    csrw mepc, t0
    mret
.endm

/* to_user label: leave supervisor mode for user mode at the label */
.macro to_user label
    li t0, 0x100
    csrc sstatus, t0
    la t0, \label
    csrw sepc, t0
    sret
.endm

/* entry table, index, target, flags: a page table entry that points at the */
/* label `target`, with the low bits `flags` */
.macro entry table, index, target, flags
    la t0, \table
    la t1, \target
    srli t1, t1, 12
    slli t1, t1, 10
    ori t1, t1, \flags
    sd t1, \index * 8(t0)
.endm

/* entry_at: the same for an address given as a number */
.macro entry_at table, index, address, flags
    la t0, \table
    li t1, \address >> 2
    ori t1, t1, \flags
    sd t1, \index * 8(t0)
.endm

/* paging_on root: the two gigabytes the examples run in mapped to */
/* themselves, devices in the first and memory in the third, then Sv39 on */
.macro paging_on root
    entry_at \root, 0, 0x00000000, 0xc7
    entry_at \root, 2, 0x80000000, 0xcf
    la t0, \root
    srli t0, t0, 12
    li t1, 8 << 60
    or t0, t0, t1
    csrw satp, t0
    sfence.vma
.endm

.macro finish
    li t0, TOHOST
    li t1, 1
    sd t1, 0(t0)
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
    li t1, 60
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
