#!/usr/bin/env bash

OPT="$HOME/.opt/scalar/llvm-bin/bin/opt"
CLANG="$HOME/.opt/scalar/llvm-bin/bin/clang"
LOCS="./loop-locs"
LOCS2="./loop-locs-fixed"


# Loop finder:

$CLANG loops.c -S -g -emit-llvm
$OPT -load-pass-plugin=../build/lib/libFindInnerLoops.so \
    -passes="print<inner-loop>" \
    -o /dev/null \
    loops.ll 2> "$LOCS"

# Loop information:
tr '\n' ' ' < "$LOCS" > "$LOCS2"

$CLANG loops.c -S -g -emit-llvm \
    -fno-slp-vectorize \
    -O3 -Rpass=loop-vectorize \
    -mllvm -debug-only=loop-vectorize \
    -o opt.ll


$OPT -load-pass-plugin=../build/lib/libInformation.so \
    -passes="print<info>" \
    -o /dev/null \
    opt.ll
