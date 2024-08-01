#!/usr/bin/env bash

clang loops.c -S -g -emit-llvm
opt -load-pass-plugin=../build/lib/libFindInnerLoops.so \
    -passes="print<inner-loop>" \
    -o /dev/null loops.ll
