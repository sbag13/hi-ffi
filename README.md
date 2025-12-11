# hi-ffi

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**hi-ffi** is a Rust procedural macro that automatically generates Foreign Function Interface (FFI) bindings for your Rust code, enabling seamless integration with C++, Swift and Python applications. With minimal annotations, you can expose Rust structs, functions, and methods to other languages without writing boilerplate FFI code.

## Features

- 🚀 **Zero Boilerplate**: Generate FFI bindings with a single `#[ffi]` attribute
- 🔧 **Flexible Configuration**: Control getter/setter generation with field-level attributes
- 🌐 **Multi-Language Support**: Generate bindings for C++ and Swift
- 📦 **Type Safety**: Maintain type safety across language boundaries
- 🎯 **Selective Export**: Choose which fields and methods to expose
- 🔒 **Memory Safe**: Automatic memory management for cross-language calls

## Installation

Add `hi-ffi` to your `Cargo.toml`:

```toml
[dependencies]
hi_ffi = { version = "0.1.0", features = ["cpp", "swift", "python"] }
```

**Note**: `hi-ffi` is a procedural macro crate. Enable the `cpp` and/or `swift`, `python` features based on your target languages.

## Quick Start

### Basic Usage

Annotate your Rust code with `#[ffi]` to generate FFI bindings:

```rust
use hi_ffi::ffi;

#[ffi]
#[derive(Default, Clone)]
struct Person {
    // Generate both getter and setter
    #[ffi(setter, getter)]
    age: i32,

    // Generate getter only (read-only)
    #[ffi(getter)]
    name: String,

    // Public fields automatically get getters and setters
    pub email: String,

    // Skip FFI generation for internal fields
    #[ffi(skip)]
    _internal_id: u64,
}

#[ffi]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
```

After building your Rust project, FFI bindings are generated in the `generated_code` directory, ready to be integrated into your C++, Swift or Python projects.

## Supported Features

### Structs

| Feature             | C++ | Swift | Python |
| ------------------- | --- | ----- | ------ |
| Primitive getters   | ✅   | ✅     | ✅      |
| Primitive setters   | ✅   | ✅     | ✅      |
| String getters      | ✅   | ✅     | ✅      |
| String setters      | ✅   | ✅     | ✅      |
| Struct getters      | ✅   | ✅     | ✅      |
| Struct setters      | ✅   | ✅     | ✅      |
| Default constructor | ✅   | ✅     | ✅      |

### Methods

| Feature             | C++ | Swift | Python |
| ------------------- | --- | ----- | ------ |
| Primitive arguments | ✅   | ✅     | ❌      |
| String arguments    | ✅   | ✅     | ❌      |
| Primitive return    | ✅   | ✅     | ❌      |
| String return       | ✅   | ✅     | ❌      |
| Struct arguments    | ✅   | ✅     | ❌      |
| Struct return       | ✅   | ✅     | ❌      |

### Static Methods

| Feature             | C++ | Swift | Python |
| ------------------- | --- | ----- | ------ |
| Primitive arguments | ✅   | ✅     | ❌      |
| String arguments    | ✅   | ✅     | ❌      |
| Primitive return    | ✅   | ✅     | ❌      |
| String return       | ✅   | ✅     | ❌      |
| Struct arguments    | ✅   | ✅     | ❌      |
| Struct return       | ✅   | ✅     | ❌      |

### Functions

| Feature             | C++ | Swift | Python |
| ------------------- | --- | ----- | ------ |
| Primitive arguments | ✅   | ✅     | ✅      |
| String arguments    | ✅   | ✅     | ✅      |
| Primitive return    | ✅   | ✅     | ✅      |
| String return       | ✅   | ✅     | ✅      |
| Struct arguments    | ✅   | ✅     | ✅      |
| Struct return       | ✅   | ✅     | ✅      |
| `&str` return       | ❌   | ❌     | ❌      |

## Language-Specific Examples

Complete working examples are available in the `tests` directory:

- **C++**: See [tests/cpp/README.md](tests/cpp/README.md) and [tests/cpp/main.cpp](tests/cpp/main.cpp)
- **Swift**: See [tests/swift/README.md](tests/swift/README.md) and [tests/swift/ModuleTest/Sources/ModuleTest/main.swift](tests/swift/ModuleTest/Sources/ModuleTest/main.swift)
- **Python**: See [tests/python/README.md](tests/python/README.md) and [tests/python/main.py](tests/python/main.py)

These examples demonstrate how to use the generated bindings in real applications.

## Architecture

`hi-ffi` is built with a modular architecture:

- **`lib.rs`** - Main entry point that orchestrates code generation
- **`translator`** - Parses Rust code into a language-agnostic intermediate representation
- **`wrapper`** - Generates Rust glue code and target language bindings
- **Language modules** (`cpp`, `swift`, `python`) - Generate language-specific code

The translation process:

1. Parse Rust code marked with `#[ffi]`
2. Extract type information and method signatures
3. Generate Rust FFI wrapper functions
4. Generate target language bindings (C++ headers, Swift code, etc.)

## Generated Code Location

After building your project, generated FFI code is placed in:

```
generated_code/
├── rust/          # Rust FFI wrapper functions
├── cpp/           # C++ headers and implementations
├── swift/         # Swift package structure
└── python_ffi/    # python package
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## Roadmap

- [ ] Documentation generation
- [ ] Publishing to crates.io

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with ❤️ using Rust's powerful procedural macro system.

---

**Note**: This project is in active development. All features are experimental and subject to change.
