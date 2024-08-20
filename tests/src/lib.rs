use hi_ffi::ffi;

use serde::Serialize;

#[ffi]
#[derive(Default, Clone, Serialize)]
struct TestStruct {
    // generate getter and setter
    #[ffi(setter, getter)]
    #[serde(skip_serializing)] // check if other attributes are preserved
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
#[derive(Default, Clone, Serialize)]
struct TestStruct2 {
    pub i32_field: i32,
}

#[ffi]
fn simple_function() {
    // println!("Rust: Simple function called"); // This line causes still reachable resources in valgrind report
}

#[ffi]
fn function_with_primitive_args(_i: i32, _b: bool) {
    // println!("Rust: Function with args called: i = {_i}, s = {_b}"); // This line causes still reachable resources in valgrind report
}

#[ffi]
fn function_with_string_arg(_s: String) {
    // println!("Rust: Function with string arg called: s = {_s}"); // This line causes still reachable resources in valgrind report
}

#[ffi]
fn function_with_primitive_and_string_arg(_i: i32, _b: bool, _s: String) {
    // println!("Rust: Function with args called: i = {_i}, s = {_b}, s = {_s}"); // This line causes still reachable resources in valgrind report
}

#[ffi]
fn function_return_primitive() -> i32 {
    42
}

#[ffi]
fn function_return_string() -> String {
    "String returned from Rust".to_string()
}

#[ffi]
fn combo_function(str1: String, str2: String, b: bool) -> String {
    // println!("{str1} {str2} {b}");
    if b {
        str1
    } else {
        str2
    }
}

// Having Drop defined causes still reachable resources in valgrind report
// impl Drop for TestStruct {
//     fn drop(&mut self) {
//         println!("Dropping TestStruct");
//     }
// }
