# Swift Integration Guide

This guide demonstrates how to use `hi-ffi` generated Swift bindings in your projects.

## Overview

After building your Rust project with `#[ffi]` annotations, Swift packages are generated in `generated_code/swift/`. The generated code includes:

- A C module (`CFfiModule`) that bridges to the Rust FFI
- A Swift module (`FfiModule`) that provides a native Swift API

## Prerequisites

- Rust toolchain (for building the Rust library)
- Swift 5.0 or later
- Clang (for compiling the C bridge)
- (Optional) `valgrind` for memory leak detection

## Building and Running the Example

### Step 1: Build the Rust Library

```bash
# From the project root
cargo build --features swift
```

This generates the Swift packages in `generated_code/swift/` and creates the Rust library in `target/debug/`.

### Step 2: Build and Run the Swift Example

```bash
# From the tests/swift directory
./test_swift.sh
```

The script:

1. Builds the Swift package
2. Runs the example executable

### Manual Build

If you prefer to build manually:

```bash
# Build the Swift package
cd ModuleTest
swift build

# Run the example
swift run
```

## Project Structure

The generated Swift code follows this structure:

```
generated_code/swift/
├── CFfiModule/          # C bridge module
│   ├── Package.swift
│   └── Sources/
│       └── CFfiModule/
│           ├── ffi_swift.h
│           └── module.modulemap
└── FfiModule/           # Swift API module
    ├── Package.swift
    └── Sources/
        └── FfiModule/
            ├── base.swift
            └── [generated Swift files]
```


## Memory Management

Swift's automatic reference counting (ARC) handles memory management:

- Struct instances are automatically managed
- String conversions are handled transparently
- No manual memory management required

## Example Code

See [main.swift](ModuleTest/Sources/ModuleTest/main.swift) for a complete working example demonstrating:

- Struct creation and manipulation
- Function calls with various argument types
- Method invocations (instance and static)
- String handling




