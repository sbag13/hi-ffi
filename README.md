# hi-ffi

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**hi-ffi** is a Rust procedural macro that automatically generates Foreign Function Interface (FFI) bindings for your Rust code, enabling seamless integration with C++, Swift, Python, and Java applications. With minimal annotations, you can expose Rust structs, functions, and methods to other languages without writing boilerplate FFI code.

## Features

- 🚀 **Zero Boilerplate**: Generate FFI bindings with a single `#[ffi]` attribute
- 🔧 **Flexible Configuration**: Control getter/setter generation with field-level attributes
- 🌐 **Multi-Language Support**: Generate bindings for C++, Swift, Python, and Java
- 📦 **Type Safety**: Maintain type safety across language boundaries
- 🎯 **Selective Export**: Choose which fields and methods to expose
- 🔒 **Memory Safe**: Automatic memory management for cross-language calls

## Installation

Add `hi-ffi` to your `Cargo.toml`:

```toml,ignore
[dependencies]
hi_ffi = { version = "0.10", features = ["cpp", "swift", "python", "java"] }
```

**Note**: `hi-ffi` is a procedural macro crate. Enable the feature or features for your target languages: `cpp`, `swift`, `python`, and/or `java`.

## Quick Start

### Basic Usage

Annotate your Rust code with `#[ffi]` to generate FFI bindings:

```rust
use hi_ffi::ffi;

// It's possible to implement traits/protocols outside Rust
#[ffi]
pub trait Shape {
    fn draw(&self, title: Option<String>);
    fn area(&self) -> f64;
    fn center_point(&self) -> Vec<f64>;
}

// A shape argument may be passed from Rust and from other languages as well
#[ffi]
pub fn take_shape(shape: Box<dyn Shape>) {
    shape.draw("Give it a title in Rust".to_string().into());
}

struct Circle {
    radius: f64,
}
impl Shape for Circle {
    fn draw(&self, title: Option<String>) {
        println!("Drawing a circle with radius {} and title {:?}", self.radius, title);
    }
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    fn center_point(&self) -> Vec<f64> {
        vec![0.0, 0.0]
    }
}

// Default can be implemented manually for a struct, and then used in other languages as well
impl Default for Circle {
    fn default() -> Self {
        Circle { radius: 1.0 }
    }
}

// PartialEq can be implemented manually for a struct, and then used in other languages as well
impl PartialEq for Circle {
    fn eq(&self, other: &Self) -> bool {
        self.radius == other.radius
    }
}

// A Rust implementation of Shape can be returned to other language,
// and then received in a function like take_shape, interchangeably
// with other language implementations of Shape
#[ffi]
pub fn create_circle(radius: f64) -> Box<dyn Shape> {
    Box::new(Circle { radius })
}

#[ffi]
#[derive(Default, Clone, PartialEq)]
struct Person {
    // Generate both getter and setter
    #[ffi(setter, getter)]
    age: i32,

    // Generate getter only (read-only)
    // i.a. Options are supported
    #[ffi(getter)]
    name: Option<String>,

    // Public fields automatically get getters and setters
    // i.a. Vectors are supported
    pub emails: Vec<String>,

    // Skip FFI generation for internal fields
    #[ffi(skip)]
    _internal_id: u64,
}

// Exposing methods
#[ffi]
impl Person {
    // only public methods are exposed
    pub fn info(&self) -> Result<Vec<String>, SimpleError> {
        Ok(vec![
            self.name.as_ref().map_or("no_name", |n| n).to_string(),
            format!("age: {}", self.age)
        ])
    }
}

#[derive(Debug)]
pub struct SimpleError;
impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SimpleError")
    }
}
// An error must implement std::error::Error to be used with ffi macro
impl std::error::Error for SimpleError {}

#[ffi]
fn greet(names: Vec<String>) -> Result<String, SimpleError> {
    Ok(format!("Hello, {:?}!", names))
}
```

After building your Rust project, FFI bindings are generated in the `generated_code` directory, ready to be integrated into your C++, Swift, Python, or Java projects.

## Language-Specific Examples

Complete working examples are available in the `tests` directory:

- **C++**: See [tests/cpp/README.md](tests/cpp/README.md) and [tests/cpp/main.cpp](tests/cpp/main.cpp)
- **Swift**: See [tests/swift/README.md](tests/swift/README.md) and [tests/swift/ModuleTest/Sources/ModuleTest/main.swift](tests/swift/ModuleTest/Sources/ModuleTest/main.swift)
- **Python**: See [tests/python/README.md](tests/python/README.md) and [tests/python/main.py](tests/python/main.py)
- **Java**: See [tests/java/Main.java](tests/java/Main.java) and [tests/java/test_java.sh](tests/java/test_java.sh)

These examples demonstrate how to use the generated bindings in real applications.

## Supported Features

hi-ffi generates bindings for Rust structs, free functions, and associated methods, including:

- Primitive and string values, structs, C-like enums, and vectors of supported values.
- Struct field getters and setters, with controls for which accessors are generated.
- Optional values (`Option<T>`) and fallible results (`Result<T, E>`) in supported function and method signatures; optional struct fields are supported as well.
- Traits and trait objects that can cross the language boundary in either direction.
- Default construction and equality support where provided by the Rust type.

Generated bindings expose idiomatic native types in the target language. For example, Java bindings use types such as `Optional` and `List`; conversions to and from Rust's FFI representations are handled by generated glue code.

## Architecture

`hi-ffi` is built with a modular architecture:

- **`lib.rs`** - Main entry point that orchestrates code generation
- **`translator`** - Parses Rust code into a language-agnostic intermediate representation
- **`wrapper`** - Generates Rust glue code and target language bindings
- **Language modules** (`cpp`, `swift`, `python`, `java`) - Generate language-specific code

The translation process:

1. Parse Rust code marked with `#[ffi]`
2. Extract type information and method signatures
3. Generate Rust FFI wrapper functions
4. Generate target language bindings (C++, Swift, Python, and Java code)

## Generated Code Location

After building your project, generated FFI code is placed in:

```ignore
generated_code/
├── rust/          # Rust FFI wrapper functions
├── cpp/           # C++ headers and implementations
├── swift/         # Swift package structure
├── java/          # Java package structure
└── python_ffi/    # python package
```

Java bindings load the native library using the Cargo package name by default, with hyphens normalized to underscores. To override it, set the `hi-ffi.library` system property to a native library name or an absolute library path.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## Roadmap

- [ ] Doc strings
- [ ] Async

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with ❤️ using Rust's powerful procedural macro system.

---

**Note**: This project is in active development. All features are experimental and subject to change.
