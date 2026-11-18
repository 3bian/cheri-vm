#!/bin/sh
# Copyright 3bian Limited and Contributors
# SPDX-License-Identifier: MIT

set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: build.sh OUTPUT-DIRECTORY" >&2
    exit 2
fi

here=$(cd "$(dirname "$0")" && pwd)
output=$(mkdir -p "$1" && cd "$1" && pwd)
commit=3df703cd52d5f0d8224e3c50bf03c6cb70efff98
source="$output/riscv-tests"

if [ ! -d "$source" ]; then
    git clone --quiet https://github.com/riscv-software-src/riscv-tests "$source"
fi
git -C "$source" checkout --quiet "$commit"
git -C "$source" submodule update --init --quiet env

build() {
    folder=$1
    width=$2
    suite=$3
    architecture=$4
    abi=$5
    mkdir -p "$output/$folder"
    for program in "$source/isa/rv${width}u${suite}"/*.S; do
        name=$(basename "$program" .S)
        if [ "$name" = ma_data ]; then
            continue
        fi
        clang --target="riscv$width-unknown-elf" "-march=$architecture" "-mabi=$abi" \
            -E -P -x assembler-with-cpp \
            "-I$here" "-I$source/isa/macros/scalar" "-I$source/env" \
            -o "$output/expanded.s" "$program"
        if [ "$width" = 32 ]; then
            sed -i 's/\(li x[0-9]*, 0x\)[0-9a-f]\{8\}\([0-9a-f]\{8\}\);/\1\2;/g' "$output/expanded.s"
        fi
        clang --target="riscv$width-unknown-elf" "-march=$architecture" "-mabi=$abi" \
            -mno-relax -nostdlib -static -fuse-ld=lld -x assembler \
            "-Wl,-T,$here/link.ld" \
            -o "$output/$folder/rv${width}u${suite}-$name.elf" "$output/expanded.s"
    done
}

privileged() {
    folder=$1
    triple=$2
    abi=$3
    architecture=$4
    shift 4
    mkdir -p "$output/$folder"
    for suite in "$@"; do
        for program in "$source/isa/$suite"/*.S; do
            name=$(basename "$program" .S)
            if [ "$suite-$name" = rv32ud-move ]; then
                continue
            fi
            clang "--target=$triple-unknown-elf" "-march=$architecture" "-mabi=$abi" \
                -E -P -x assembler-with-cpp \
                "-I$source/env/p" "-I$source/env" "-I$source/isa/macros/scalar" \
                -o "$output/expanded.s" "$program"
            for handler in mtvec_handler stvec_handler; do
                if grep -q "\.globa*l $handler" "$output/expanded.s"; then
                    sed -i "s/\.weak $handler;//" "$output/expanded.s"
                fi
            done
            sed -i 's/\btcontrol\b/0x7a5/g; s/\btselect\b/0x7a0/g; s/\btdata1\b/0x7a1/g; s/\btdata2\b/0x7a2/g' \
                "$output/expanded.s"
            clang "--target=$triple-unknown-elf" "-march=$architecture" "-mabi=$abi" \
                -mno-relax -nostdlib -static -fuse-ld=lld -x assembler \
                "-Wl,-T,$source/env/p/link.ld" \
                -o "$output/$folder/$suite-$name.elf" "$output/expanded.s"
        done
    done
    rm -f "$output/expanded.s"
}

virtual() {
    folder=$1
    architecture=$2
    shift 2
    mkdir -p "$output/$folder" "$output/environment"
    common="--target=riscv64-unknown-elf -mabi=lp64 -mcmodel=medany -mno-relax -ffreestanding -msmall-data-limit=0 -fno-builtin -nostdlib -static -O2 -DENTROPY=0x1234567 -Wno-everything -I$here/include -I$source/env/v -I$source/env -I$source/isa/macros/scalar"
    for part in string.c vm.c entry.S; do
        clang $common -march=rv64imafdc_zicsr_zifencei -c \
            -o "$output/environment/$part.o" "$source/env/v/$part"
    done
    for suite in "$@"; do
        for program in "$source/isa/$suite"/*.S; do
            name=$(basename "$program" .S)
            if [ "$name" = ma_data ]; then
                continue
            fi
            clang $common "-march=$architecture" -fuse-ld=lld "-Wl,-T,$source/env/v/link.ld" \
                -o "$output/$folder/$suite-$name.elf" "$program" "$output/environment"/*.o
        done
    done
}

build rv32 32 i rv32i_zicsr_zifencei ilp32
build rv32 32 m rv32im_zicsr_zifencei ilp32
build rv32 32 a rv32ia_zicsr_zifencei ilp32
build rv32 32 c rv32ic_zicsr_zifencei ilp32
build rv64 64 i rv64i_zicsr_zifencei lp64
build rv64 64 m rv64im_zicsr_zifencei lp64
build rv64 64 a rv64ia_zicsr_zifencei lp64
build rv64 64 c rv64ic_zicsr_zifencei lp64

build rv32-bits 32 zba rv32i_zba_zicsr_zifencei ilp32
build rv32-bits 32 zbb rv32i_zbb_zicsr_zifencei ilp32
build rv32-bits 32 zbc rv32i_zbc_zicsr_zifencei ilp32
build rv32-bits 32 zbs rv32i_zbs_zicsr_zifencei ilp32
build rv32-bits 32 zbkb rv32i_zbkb_zicsr_zifencei ilp32
build rv32-bits 32 zbkx rv32i_zbkx_zicsr_zifencei ilp32
build rv32-bits 32 zicond rv32i_zicond_zicsr_zifencei ilp32
build rv64-bits 64 zba rv64i_zba_zicsr_zifencei lp64
build rv64-bits 64 zbb rv64i_zbb_zicsr_zifencei lp64
build rv64-bits 64 zbc rv64i_zbc_zicsr_zifencei lp64
build rv64-bits 64 zbs rv64i_zbs_zicsr_zifencei lp64
build rv64-bits 64 zbkb rv64i_zbkb_zicsr_zifencei lp64
build rv64-bits 64 zbkx rv64i_zbkx_zicsr_zifencei lp64
build rv64-bits 64 zicond rv64i_zicond_zicsr_zifencei lp64

privileged rv64-privileged riscv64 lp64 rv64imafdc_zicsr_zifencei rv64mi rv64si
privileged rv64-float riscv64 lp64 rv64imafdc_zicsr_zifencei rv64uf rv64ud
privileged rv32-float riscv32 ilp32 rv32imafdc_zicsr_zifencei rv32uf rv32ud
virtual rv64-virtual rv64imafdc_zicsr_zifencei rv64ui rv64um rv64ua rv64uc rv64uf rv64ud

echo "RISCV_TEST_BINARIES=$output/rv32"
echo "RISCV64_TEST_BINARIES=$output/rv64"
echo "RISCV64_PRIVILEGED_TEST_BINARIES=$output/rv64-privileged"
echo "RISCV64_VIRTUAL_TEST_BINARIES=$output/rv64-virtual"
echo "RISCV64_FLOAT_TEST_BINARIES=$output/rv64-float"
echo "RISCV_FLOAT_TEST_BINARIES=$output/rv32-float"
echo "RISCV_BIT_TEST_BINARIES=$output/rv32-bits"
echo "RISCV64_BIT_TEST_BINARIES=$output/rv64-bits"
