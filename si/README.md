# Scalar Interpolation

Programs used by the scalar interpolation crawler interface.

## Build Instructions

All programs can be built using the following commands:

``` bash
mkdir build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Debug \
         -DLT_LLVM_INSTALL_DIR=/path/to/llvm/install/dir
make
```

## Find Inner Loops

The LLVM pass in the `find_inner_loops` directory prints a list of `(line,
column)`, where each entry corresponds to an innermost loop.

