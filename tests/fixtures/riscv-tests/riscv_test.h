#ifndef _ENV_STOP_H
#define _ENV_STOP_H
#define RVTEST_RV32U
#define RVTEST_RV64U
#define RVTEST_RV32M
#define RVTEST_RV64M
#define TESTNUM gp
#define RVTEST_CODE_BEGIN .section .text.init; .globl _start; _start:
#define RVTEST_CODE_END
#define RVTEST_PASS li a0, 0; ebreak
#define RVTEST_FAIL li a0, 1; ebreak
#define EXTRA_DATA
#define RVTEST_DATA_BEGIN .data; .align 4; .globl begin_signature; begin_signature:
#define RVTEST_DATA_END .align 4; .globl end_signature; end_signature:
#endif
