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
kernel_tag=v6.12
firmware_commit=ed70256dd22c09962d146e433834239825ef0616
library_commit=0784374d561435f7c787a555aeab8ede699ed298
runtime_tag=llvmorg-18.1.3
shell_tag=1_36_1
kernel="$output/linux"
firmware="$output/opensbi"
library="$output/musl"
runtime="$output/llvm"
shell="$output/busybox"
root="$output/sysroot"
tree="$here/../../../profiles/cva6.dts"
target="--target=riscv64-linux-musl -march=rv64gc -mabi=lp64d"
applets="cat echo grep head kill ls seq sleep sort test true uname wc"

if [ ! -d "$kernel" ]; then
    git clone --quiet --depth 1 --branch "$kernel_tag" https://github.com/torvalds/linux "$kernel"
fi
if [ ! -d "$firmware" ]; then
    git clone --quiet https://github.com/riscv-software-src/opensbi "$firmware"
fi
git -C "$firmware" checkout --quiet "$firmware_commit"
if [ ! -d "$library" ]; then
    git clone --quiet https://github.com/kraj/musl "$library"
fi
git -C "$library" checkout --quiet "$library_commit"
if [ ! -d "$runtime" ]; then
    git clone --quiet --depth 1 --branch "$runtime_tag" --filter=blob:none --sparse \
        https://github.com/llvm/llvm-project "$runtime"
    git -C "$runtime" sparse-checkout set compiler-rt/lib/builtins
fi
if [ ! -d "$shell" ]; then
    git clone --quiet --depth 1 --branch "$shell_tag" https://github.com/mirror/busybox "$shell"
fi

(cd "$library" && CC="clang $target" AR=llvm-ar RANLIB=llvm-ranlib ./configure \
    --target=riscv64-linux-musl "--prefix=$root" --disable-shared) > "$output/library.log" 2>&1
make -C "$library" -j"$(nproc)" install >> "$output/library.log" 2>&1

builtins="$runtime/compiler-rt/lib/builtins"
mkdir -p "$output/runtime"
for routine in addtf3 comparetf2 divtf3 extenddftf2 extendsftf2 fixtfdi fixtfsi fixtfti \
    fixunstfdi fixunstfsi fixunstfti floatditf floatsitf floattitf floatunditf floatunsitf \
    floatuntitf multf3 subtf3 trunctfdf2 trunctfsf2 clzdi2 clzsi2 ctzdi2 ctzsi2 popcountdi2 \
    popcountsi2 bswapdi2 bswapsi2 multi3 udivti3 umodti3 divti3 modti3 udivmodti4 ashlti3 \
    ashrti3 lshrti3 ffsdi2 paritydi2 riscv/fp_mode; do
    clang $target -O2 -ffreestanding -fno-stack-protector -fno-pic "--sysroot=$root" \
        "-I$builtins" -c "$builtins/$routine.c" -o "$output/runtime/$(basename "$routine").o"
done
rm -f "$root/lib/libgcc.a" "$root/lib/libgcc_eh.a"
llvm-ar rcs "$root/lib/libgcc.a" "$output"/runtime/*.o
llvm-ar rcs "$root/lib/libgcc_eh.a"
: > "$output/runtime/empty.c"
for object in crtbegin crtbeginT crtend; do
    clang $target -c "$output/runtime/empty.c" -o "$root/lib/$object.o"
done

compiler="clang $target --sysroot=$root -fuse-ld=lld"
$compiler -static -O2 -o "$output/init" "$here/init.c"
$compiler -static -O2 -o "$output/pi" "$here/pi.c"

make -C "$shell" allnoconfig > "$output/shell.log" 2>&1
while read -r option; do
    name=${option%%=*}
    sed -i "s/^# $name is not set\$/$option/" "$shell/.config"
done < "$here/busybox.config"
yes "" | make -C "$shell" oldconfig >> "$output/shell.log" 2>&1
make -C "$shell" -j"$(nproc)" "CC=$compiler" HOSTCC=cc AR=llvm-ar LD=ld.lld STRIP=llvm-strip \
    busybox >> "$output/shell.log" 2>&1

{
    cat <<LIST
dir /dev 0755 0 0
nod /dev/console 0600 0 0 c 5 1
nod /dev/null 0666 0 0 c 1 3
nod /dev/ttyS0 0600 0 0 c 4 64
dir /bin 0755 0 0
file /init $output/init 0755 0 0
file /bin/pi $output/pi 0755 0 0
file /bin/busybox $shell/busybox 0755 0 0
slink /bin/sh busybox 0777 0 0
LIST
    for applet in $applets; do
        echo "slink /bin/$applet busybox 0777 0 0"
    done
} > "$kernel/initramfs.list"

make -C "$kernel" ARCH=riscv LLVM=1 tinyconfig > "$output/kernel.log" 2>&1
(cd "$kernel" && ARCH=riscv LLVM=1 scripts/kconfig/merge_config.sh -m .config "$here/linux.config") \
    >> "$output/kernel.log" 2>&1
make -C "$kernel" ARCH=riscv LLVM=1 olddefconfig >> "$output/kernel.log" 2>&1
make -C "$kernel" ARCH=riscv LLVM=1 -j"$(nproc)" Image >> "$output/kernel.log" 2>&1

if command -v dtc > /dev/null; then
    dtc -I dts -O dtb -o "$output/cva6.dtb" "$tree"
else
    pydtc pack -v 17 -o "$output/cva6.dtb" "$tree"
fi

make -C "$firmware" PLATFORM=generic LLVM=1 "O=$output/firmware" "FW_FDT_PATH=$output/cva6.dtb" \
    FW_TEXT_START=0x80000000 FW_PIC=n FW_JUMP=n FW_DYNAMIC=n \
    "FW_PAYLOAD_PATH=$kernel/arch/riscv/boot/Image" FW_PAYLOAD_OFFSET=0x200000 \
    > "$output/firmware.log" 2>&1

echo "LINUX_FIRMWARE=$output/firmware/platform/generic/firmware/fw_payload.bin"
