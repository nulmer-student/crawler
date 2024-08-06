#!/usr/bin/env bash

OPT="$HOME/.opt/scalar/llvm-bin/bin/opt"

clang loops.c -S -g -emit-llvm
$OPT -load-pass-plugin=../build/lib/libFindInnerLoops.so \
    -passes="print<inner-loop>" \
    -o /dev/null \
    loops.ll
