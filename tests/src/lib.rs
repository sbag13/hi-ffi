use hi_ffi::ffi;

use serde::Serialize;

#[ffi]
#[derive(Default, Clone, Serialize, Debug)]
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

// TODO
#[ffi]
impl TestStruct {
    //     #[ffi(constructor)]
    //     fn new() -> Self {
    //         Self {
    //             i32_field: 42,
    //             bool_field: true,
    //             string_field: "Hello from Rust".to_string(),
    //             _skip_field: 0,
    //             struct_field: TestStruct2::default(),
    //         }
    //     }

    // TODO static method

    pub fn public_method(&self) {}

    // private method is skipped
    fn private_method(&self) {}

    //     pub fn public_method_taking_primitives(&self, i: i32, b: bool) {
    //         println!("Rust: Public method called: i = {i}, b = {b}");
    //     }

    //     pub fn public_method_taking_string(&self, s: String) {
    //         println!("Rust: Public method called: s = {s}");
    //     }

    //     pub fn public_method_taking_struct(&self, s: TestStruct2) {
    //         println!("Rust: Public method called: s = {s:?}");
    //     }

    //     pub fn public_method_returning_primitive(&self) -> i32 {
    //         42
    //     }

    //     pub fn public_method_returning_string(&self) -> String {
    //         "String returned from Rust method".to_string()
    //     }

    //     pub fn public_method_returning_struct(&self) -> TestStruct2 {
    //         TestStruct2::default()
    //     }

    //     pub fn combo_method(&self, str1: String, str2: String, b: bool, obj: TestStruct2) -> String {
    //         println!("{str1} {str2} {b} {obj:?}");
    //         if b {
    //             str1
    //         } else {
    //             str2
    //         }
    //     }
}

#[ffi]
#[derive(Default, Clone, Serialize, Debug)]
struct TestStruct2 {
    pub i32_field: i32,
}

#[ffi]
fn simple_function() {
    println!("Rust: Simple function called"); // This line causes still reachable resources in valgrind report
}

#[ffi]
fn function_with_primitive_args(i: i32, b: bool) {
    println!("Rust: Function with args called: i = {i}, s = {b}"); // This line causes still reachable resources in valgrind report
}

#[ffi]
fn function_with_string_arg(s: String) {
    println!("Rust: Function with string arg called: s = {s}"); // This line causes still reachable resources in valgrind report
}

#[ffi]
fn function_with_primitive_and_string_arg(i: i32, b: bool, s: String) {
    println!("Rust: Function with args called: i = {i}, s = {b}, s = {s}"); // This line causes still reachable resources in valgrind report
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
impl Drop for TestStruct {
    fn drop(&mut self) {
        println!("Dropping TestStruct");
    }
}
