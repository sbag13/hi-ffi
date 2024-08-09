# Stick To Rust

Tool for generating bindings from Rust code.

## How To Use

Rust glue code and bindings in target languages are generated for Rust code that is marked with the `#[ffi]` macro.

```rust
#[ffi]
use stick_to_rust::ffi;

#[ffi]
#[derive(Default, Clone)]
struct TestStruct {
    // generate getter and setter
    #[ffi(setter, getter)]
    i32_field: i32,

    // generate getter only
    #[ffi(getter)]
    bool_field: bool,

    // generate getter and setter
    pub string_field: String,

    // don't generate getter and setter
    #[ffi(skip)]
    _skip_field: i32,

    #[ffi(getter, setter)]
    struct_field: TestStruct2,
}

#[ffi]
#[derive(Default, Clone)]
struct TestStruct2 {
    pub i32_field: i32,
}

#[ffi]
fn function_with_primitive_and_string_arg(_i: i32, _b: bool, _s: String) {}
```

After building Rust package, the FFI code is generated in `generated_code` directory, which then can be embedded into a program written in one of target languages.

### Language specific examples:

- C++ - [README.md](tests/cpp/README.md)
- Swift - [README.md](tests/swift/README.md)

## Supported Features

|           |                          | c++ | swift |
| --------- | ------------------------ | --- | ----- |
| structs   | primitive setters        | ✅   | ✅     |
|           | primitive getters        | ✅   | ✅     |
|           | string setters           | ✅   | ❌     |
|           | string getters           | ✅   | ❌     |
|           | struct getters           | ✅   | ❌     |
|           | struct setters           | ✅   | ❌     |
|           | default constructor      | ✅   | ✅     |
|           |
| methods   | primitive arguments      | ❌   | ❌     |
|           | string arguments         | ❌   | ❌     |
|           | primitive return         | ❌   | ❌     |
|           | string return            | ❌   | ❌     |
|           | struct arguments         | ❌   | ❌     |
|           | struct return            | ❌   | ❌     |
|           |
| functions | primitive arguments      | ✅   | ✅     |
|           | string arguments         | ✅   | ✅     |
|           | primitive return         | ✅   | ✅     |
|           | string return            | ✅   | ✅     |
|           | str return               | ❌   | ❌     |
|           | struct arguments         | ❌   | ❌     |
|           | struct return            | ❌   | ❌     |
|           |
| enums     | primitive enums          | ❌   | ❌     |
|           | variants with primitives | ❌   | ❌     |
|           | variants with strings    | ❌   | ❌     |
|           | variants with structs    | ❌   | ❌     |
|           |
| vector    | primitives vector        | ❌   | ❌     |
|           | strings vector           | ❌   | ❌     |

## Development

### Modules

`stick_to_rust` contains of the following modules:

- `lib.rs` - main module, creates target files and directories, calls `translator` and `wrapper` modules
- `translator` - translates Rust code into parsed, intermediate representation; it is target language agnostic
- `wrapper` - result of translation. it is used for generating Rust glue code (`Into<TokenStream>`) and bindings for every target language
- language specific base modules (`cpp`, `swift`) - generates base code for each target language

# TODO

- cpp tests instead of main
- push to github
- pipelines
  - publishing to crates.io
