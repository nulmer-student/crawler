# Scalar Interpolation

Programs used by the scalar interpolation crawler interface.

## Build Instructions

All programs can be built using the following commands:

``` bash
export LLVM_INSTALL_DIR="/path/to/llvm/bin"
```

## Building the Cost Model

The cost model is built with the following commands:

``` bash

cmake ../llvm \
    -GNinja \
    -DLLVM_OPTIMIZED_TABLEGEN=ON \
    -DLLVM_INSTALL_UTILS=ON \
    -DBUILD_SHARED_LIBS=ON \
    -DLLVM_ENABLE_RTTI=ON \
    -DLLVM_ENABLE_ASSERTIONS=ON \
    -DLLVM_INCLUDE_BENCHMARKS=OFF \
    -DLLVM_CCACHE_BUILD=OFF \
    -DLLVM_USE_NEWPM=ON \
    -DLLVM_PARALLEL_LINK_JOBS=2 \
    -DLLVM_TARGETS_TO_BUILD="X86" \
    -DCMAKE_BUILD_TYPE=Release \
    -DLLVM_ENABLE_PROJECTS="clang;clang-tools-extra;lld" \
    -DLLVM_USE_LINKER=lld \
    -DCMAKE_C_COMPILER=clang \
    -DCMAKE_CXX_COMPILER=clang++
```
