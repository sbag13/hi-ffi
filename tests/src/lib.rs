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
    #[ffi(getter, setter)]
    bool_field: bool,

    // generate getter and setter
    pub string_field: String,

    // skip setter
    #[ffi(getter)]
    _no_setter: i32,

    pub f32_field: f32,

    // don't generate getter and setter
    #[ffi(skip)]
    _skip_field: i32,

    #[ffi(getter, setter)]
    struct_field: TestStruct2,
}

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

    pub fn public_method(&self) {
        // println!("Rust: Public method called");
    }

    // private method is skipped
    #[allow(dead_code)]
    fn private_method(&self) {}

    pub fn public_method_taking_primitives(&self, _i: i32, _b: bool) {
        // println!("Rust: Public method called: i = {_i}, b = {_b}");
    }

    pub fn public_method_taking_string(&self, _s: String) {
        // println!("Rust: Public method called: s = {_s}");
    }

    pub fn public_method_taking_struct(&self, _s: TestStruct2) {
        // println!("Rust: Public method called: s = {_s:?}");
    }

    pub fn public_method_returning_primitive(&self) -> i32 {
        24
    }

    pub fn public_method_returning_bool(&self) -> bool {
        true
    }

    pub fn public_method_returning_string(&self) -> String {
        "String returned from Rust method".to_string()
    }

    pub fn public_method_returning_struct(&self) -> TestStruct2 {
        TestStruct2 { i32_field: 99 }
    }

    pub fn combo_method(&self, str1: String, str2: String, b: bool) -> String {
        // println!("{str1} {str2} {b}");
        if b { str1 } else { str2 }
    }

    // static method
    pub fn static_method() {
        // println!("Rust: Static public method called");
    }
    pub fn static_method_taking_primitives(_i: i32, _b: bool) {
        // println!("Rust: Static public method called: i = {_i}, b = {_b}");
    }
    pub fn static_method_taking_string(_s: String) {
        // println!("Rust: Static public method called: s = {_s}");
    }
    pub fn static_method_taking_struct(_s: TestStruct2) {
        // println!("Rust: Static public method called: s = {_s:?}");
    }
    pub fn static_method_returning_primitive() -> i32 {
        22
    }
    pub fn static_method_returning_string() -> String {
        "String returned from Rust static method".to_string()
    }
    pub fn static_method_returning_struct() -> TestStruct2 {
        TestStruct2 { i32_field: 77 }
    }
    pub fn static_combo_method(str1: String, str2: String, b: bool) -> String {
        // println!("{str1} {str2} {b}");
        if b { str1 } else { str2 }
    }
    pub fn static_combo_struct_method(s1: TestStruct2, _s2: TestStruct2) -> TestStruct2 {
        // println!("Rust: Static combo struct method: s1 = {s1:?}, s2 = {_s2:?}");
        s1
    }
}

#[ffi]
#[derive(Default, Clone, Serialize, Debug)]
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
fn function_return_float() -> f64 {
    5.21
}

#[ffi]
fn function_return_negated_bool(input: bool) -> bool {
    !input
}

#[ffi]
fn function_return_string() -> String {
    "String returned from Rust".to_string()
}

#[ffi]
fn combo_function(str1: String, str2: String, b: bool, _s: TestStruct) -> String {
    // println!("{str1} {str2} {b} {_s:?}");
    if b { str1 } else { str2 }
}

#[ffi]
fn function_taking_struct(_s: TestStruct2) {
    // println!("Rust: Function with struct arg called: s = {_s:?}");
}

#[ffi]
fn function_returning_struct() -> TestStruct2 {
    TestStruct2 { i32_field: 48 }
}

#[ffi]
fn combo_struct_function(s1: TestStruct, _s2: TestStruct, _s3: TestStruct2) -> TestStruct {
    // println!("Rust: Combo struct function called: s1 = {s1:?}, s2 = {_s2:?}, s3 = {_s3:?}");
    s1
}

#[ffi]
fn function_taking_vec_of_primitives(_vec: Vec<i32>) {
    // println!(
    //     "Rust: Function with vector of primitives called: {:?}",
    //     _vec
    // );
}

#[ffi]
fn function_taking_vec_of_bools(_vec: Vec<bool>) {
    // println!("Rust: Function with vector of bools called: {:?}", _vec);
}

#[ffi]
fn function_taking_vec_of_strings(_vec: Vec<String>) {
    // println!("Rust: Function with vector of strings called: {:?}", _vec);
}

#[ffi]
fn function_taking_vec_of_structs(_vec: Vec<TestStruct>) {
    // println!("Rust: Function with vector of structs called: {:?}", _vec);
}

// Having Drop defined causes still reachable resources in valgrind report
impl Drop for TestStruct {
    fn drop(&mut self) {
        // println!("Dropping TestStruct");
    }
}

impl Drop for TestStruct2 {
    fn drop(&mut self) {
        // println!("Dropping TestStruct2");
    }
}
