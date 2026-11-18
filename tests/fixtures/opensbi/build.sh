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
commit=ed70256dd22c09962d146e433834239825ef0616
source="$output/opensbi"
tree="$here/../../../profiles/cva6.dts"

if [ ! -d "$source" ]; then
    git clone --quiet https://github.com/riscv-software-src/opensbi "$source"
fi
git -C "$source" checkout --quiet "$commit"

if command -v dtc > /dev/null; then
    dtc -I dts -O dtb -o "$output/cva6.dtb" "$tree"
else
    pydtc pack -v 17 -o "$output/cva6.dtb" "$tree"
fi

make -C "$source" PLATFORM=generic LLVM=1 "FW_FDT_PATH=$output/cva6.dtb" \
    FW_TEXT_START=0x80000000 FW_PIC=n FW_PAYLOAD=n FW_DYNAMIC=n FW_JUMP_ADDR=0x80200000 \
    > "$output/build.log" 2>&1

echo "OPENSBI_FIRMWARE=$source/build/platform/generic/firmware/fw_jump.bin"
