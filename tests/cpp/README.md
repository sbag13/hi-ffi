# C++ Integration Guide

This guide demonstrates how to use `hi-ffi` generated C++ bindings in your projects.

## Overview

After building your Rust project with `#[ffi]` annotations, C++ headers and implementations are generated in `generated_code/cpp/`. These files provide a clean C++ API that wraps the Rust FFI functions.

## Prerequisites

- Rust toolchain (for building the Rust library)
- C++ compiler (GCC, Clang, or MSVC)
- `g++` or equivalent C++ compiler
- (Optional) `valgrind` for memory leak detection

## Building and Running the Example

### Step 1: Build the Rust Library

```bash
# From the project root
cargo build
```

This generates the FFI bindings in `generated_code/cpp/` and creates the Rust library in `target/debug/`.

### Step 2: Compile and Run the C++ Example

```bash
# From the tests/cpp directory
./test_cpp.sh
```

The script:

1. Compiles all generated C++ files with `main.cpp`
2. Links against the Rust library
3. Runs the executable

### Manual Compilation

If you prefer to compile manually:

```bash
g++ ../generated_code/cpp/*.cpp main.cpp \
    -I ../generated_code/cpp/ \
    -L ../target/debug/ \
    -l tests \
    -o test && \
./test
```

## Memory Management

The generated C++ classes handle memory management automatically:

- Structs are reference-counted and automatically cleaned up
- String conversions are handled transparently
- No manual memory management required

## Example Code

See [main.cpp](main.cpp) for a complete working example demonstrating:

- Struct creation and manipulation
- Function calls with various argument types
- Method invocations (instance and static)
- String handling

