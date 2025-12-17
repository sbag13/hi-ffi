# Python Integration Guide

This guide demonstrates how to use `hi-ffi` generated Python bindings in your projects.

## Overview

After building your Rust project with `#[ffi]` annotations, Python modules are generated in `generated_code/python_ffi/`. These files provide a clean Python API that wraps the Rust FFI functions using ctypes.

## Prerequisites

- Rust toolchain (for building the Rust library)
- Python 3.7 or later
- `ctypes` module (included with Python standard library)

## Building and Running the Example

### Step 1: Build the Rust Library

```bash
# From the project root
cargo build
```

This generates the FFI bindings in `generated_code/python_ffi/` and creates the Rust library in `target/debug/`.

### Step 2: Run the Python Example

```bash
# From the tests/python directory
python main.py
```

The script:

1. Loads the generated Python modules from `../generated_code/python_ffi/`
2. Loads the Rust library using ctypes
3. Initializes the FFI
4. Runs comprehensive tests of all functionality

### Manual Setup

If you prefer to set up the Python environment manually:

```python
import ctypes
import sys
import os

# Add generated code to path
sys.path.insert(0, os.path.abspath("../generated_code"))

# Import generated modules
from python_ffi import ffi_init
from python_ffi import *
from python_ffi.TestStruct import TestStruct

# Load the Rust library
ext = "so"  # Change to 'dylib' for MacOS, 'dll' for Windows
lib_path = os.path.join("..", "target", "debug", f"libtests.{ext}")
lib = ctypes.CDLL(lib_path)

# Initialize FFI
ffi_init(lib)
```