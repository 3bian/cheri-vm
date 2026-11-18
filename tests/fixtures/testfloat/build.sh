#!/bin/sh
# Copyright 3bian Limited and Contributors
# SPDX-License-Identifier: MIT

set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: build.sh OUTPUT-DIRECTORY" >&2
    exit 2
fi

output=$(mkdir -p "$1" && cd "$1" && pwd)
arithmetic=07ebd29db0cfb55723a79fbbffe99103ef38e305
tester=a9c849f1b0eb0264b626d9686ffae167d996e3be

fetch() {
    if [ ! -d "$output/$1" ]; then
        git clone --quiet "https://github.com/ucb-bar/$1" "$output/$1"
    fi
    git -C "$output/$1" checkout --quiet "$2"
    make -C "$output/$1/build/Linux-x86_64-GCC" SPECIALIZE_TYPE=RISCV > "$output/$1.log" 2>&1
}

fetch berkeley-softfloat-3 "$arithmetic"
fetch berkeley-testfloat-3 "$tester"

generator="$output/berkeley-testfloat-3/build/Linux-x86_64-GCC/testfloat_gen"
mkdir -p "$output/vectors"

for mode in near_even minMag min max near_maxMag; do
    for width in 32 64; do
        for operation in add sub mul div sqrt eq lt le \
            to_i32 to_ui32 to_i64 to_ui64; do
            "$generator" -level 1 "-r$mode" -tininessafter -exact "f${width}_$operation" \
                > "$output/vectors/f${width}_$operation.$mode.txt"
        done
        "$generator" -level 1 "-r$mode" -tininessafter "f${width}_mulAdd" 2> /dev/null \
            | head -n 1000000 > "$output/vectors/f${width}_mulAdd.$mode.txt" || true
        for source in i32 ui32 i64 ui64; do
            "$generator" -level 1 "-r$mode" "${source}_to_f$width" \
                > "$output/vectors/${source}_to_f$width.$mode.txt"
        done
    done
    "$generator" -level 1 "-r$mode" -tininessafter f32_to_f64 > "$output/vectors/f32_to_f64.$mode.txt"
    "$generator" -level 1 "-r$mode" -tininessafter f64_to_f32 > "$output/vectors/f64_to_f32.$mode.txt"
done

echo "TESTFLOAT_VECTORS=$output/vectors"
