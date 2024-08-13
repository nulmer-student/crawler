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

$CLANG loops.c -S -g -emit-llvm -o opt.ll \
    -fno-slp-vectorize \
    -O3 -Rpass=loop-vectorize
    # -mllvm -debug-only=loop-vectorize

tr '\n' ' ' < "$LOCS" > "$LOCS2"

$OPT -load-pass-plugin=../build/lib/libInformation.so \
    -passes="print<info>" \
    --loop-locs="$(cat "$LOCS2")" \
    -o /dev/null \
    opt.ll
