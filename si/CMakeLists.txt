cmake_minimum_required(VERSION 3.20)
project(matcher)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_EXPORT_COMPILE_COMMANDS 1)

# LLVM:

find_package(LLVM REQUIRED CONFIG)

separate_arguments(LLVM_DEFINITIONS NATIVE_COMMAND ${LLVM_DEFINITIONS})
add_definitions(${LLVM_DEFINITIONS_LIST})
include_directories(${LLVM_INCLUDE_DIRS})

# Project Source:

set(CMAKE_LIBRARY_OUTPUT_DIRECTORY "${PROJECT_BINARY_DIR}/lib")
add_subdirectory(find_inner_loops)
