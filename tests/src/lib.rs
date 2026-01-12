use hi_ffi::ffi;

use serde::Serialize;

#[ffi]
#[derive(Default, Clone, Serialize, Debug, PartialEq)]
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

    pub fn public_method_taking_vec_of_primitives(&self, vec: Vec<i32>) {
        assert_eq!(vec![1, 2, 3, 4, 5], vec);
    }

    pub fn public_method_taking_vec_of_strings(&self, vec: Vec<String>) {
        assert_eq!(vec!["Hello".to_string(), "World".to_string()], vec);
    }

    pub fn public_method_taking_vec_of_structs(&self, vec: Vec<TestStruct2>) {
        let expected = vec![TestStruct2 { i32_field: 42 }, TestStruct2 { i32_field: 24 }];
        assert_eq!(expected, vec);
    }

    pub fn static_method_taking_vec_of_primitives(vec: Vec<i32>) {
        assert_eq!(vec![6, 7, 8, 9, 10], vec);
    }

    pub fn static_method_taking_vec_of_strings(vec: Vec<String>) {
        assert_eq!(vec!["Static".to_string(), "Method".to_string()], vec);
    }

    pub fn static_method_taking_vec_of_structs(vec: Vec<TestStruct2>) {
        let expected = vec![
            TestStruct2 { i32_field: 100 },
            TestStruct2 { i32_field: 200 },
        ];
        assert_eq!(expected, vec);
    }

    pub fn public_method_returning_vec_of_primitives(&self) -> Vec<i32> {
        vec![10, 20, 30, 40, 50]
    }

    pub fn public_method_returning_vec_of_strings(&self) -> Vec<String> {
        vec![
            "Method".to_string(),
            "Vector".to_string(),
            "Return".to_string(),
        ]
    }

    pub fn public_method_returning_vec_of_structs(&self) -> Vec<TestStruct2> {
        vec![
            TestStruct2 { i32_field: 300 },
            TestStruct2 { i32_field: 400 },
        ]
    }

    pub fn static_method_returning_vec_of_primitives() -> Vec<i32> {
        vec![60, 70, 80, 90, 100]
    }

    pub fn static_method_returning_vec_of_strings() -> Vec<String> {
        vec![
            "Static".to_string(),
            "Method".to_string(),
            "Vector".to_string(),
        ]
    }

    pub fn static_method_returning_vec_of_structs() -> Vec<TestStruct2> {
        vec![
            TestStruct2 { i32_field: 500 },
            TestStruct2 { i32_field: 600 },
        ]
    }
}

#[ffi]
#[derive(Default, Clone, Serialize, Debug, PartialEq)]
pub struct TestStruct2 {
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
fn function_taking_vec_of_primitives(vec: Vec<i32>) {
    // println!(
    //     "Rust: Function with vector of primitives called: {:?}",
    //     vec
    // );
    assert_eq!(vec![1, 2, 3, 4, 5], vec);
}

#[ffi]
fn function_taking_vec_of_bools(vec: Vec<bool>) {
    // println!("Rust: Function with vector of bools called: {:?}", _vec);
    assert_eq!(vec![true, false, true, true], vec);
}

#[ffi]
fn function_taking_vec_of_strings(vec: Vec<String>) {
    // println!("Rust: Function with vector of strings called: {:?}", _vec);
    assert_eq!(
        vec![
            "Hello, Rust!",
            "Hello, C++!",
            "Hello, Python!",
            "Hello, Swift!"
        ],
        vec
    );
}

#[ffi]
fn function_taking_vec_of_structs(vec: Vec<TestStruct>) {
    // println!("Rust: Function with vector of structs called: {:?}", vec);
    let mut s1 = TestStruct::default();
    s1.i32_field = 15;
    let mut s2 = TestStruct::default();
    s2.i32_field = 17;
    let expected = vec![s1, s2];
    assert_eq!(expected, vec);
}

#[ffi]
fn function_returning_vec_of_int() -> Vec<i32> {
    vec![3, 2, 7, 8]
}

#[ffi]
fn function_returning_vec_of_bool() -> Vec<bool> {
    vec![true, false, true, true]
}

#[ffi]
fn function_returning_vec_of_structs() -> Vec<TestStruct2> {
    vec![TestStruct2 { i32_field: 8 }, TestStruct2 { i32_field: 11 }]
}

#[ffi]
fn function_returning_vec_of_string() -> Vec<String> {
    vec!["Hello".to_string(), "World".to_string(), "Rust".to_string()]
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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[ffi]
pub enum TestStatus {
    Active = 1,
    Inactive = 2,
    Pending = 3,
}

#[ffi]
pub fn get_status() -> TestStatus {
    TestStatus::Active
}

#[ffi]
pub fn get_inactive_status() -> TestStatus {
    TestStatus::Inactive
}

#[ffi]
pub fn assert_active(status: TestStatus) {
    assert_eq!(status, TestStatus::Active);
}

#[ffi]
pub fn assert_inactive(status: TestStatus) {
    assert_eq!(status, TestStatus::Inactive);
}

#[ffi]
pub fn function_taking_vec_of_enums(vec: Vec<TestStatus>) {
    assert_eq!(
        vec![
            TestStatus::Active,
            TestStatus::Inactive,
            TestStatus::Pending,
            TestStatus::Active
        ],
        vec
    );
}

#[ffi]
pub fn function_returning_vec_of_enums() -> Vec<TestStatus> {
    vec![
        TestStatus::Pending,
        TestStatus::Active,
        TestStatus::Inactive,
        TestStatus::Active,
    ]
}

#[ffi]
#[derive(Debug, Clone, Default)]
pub struct StructWithVecs {
    pub vec_of_ints: Vec<i32>,
    pub vec_of_bools: Vec<bool>,
    pub vec_of_strings: Vec<String>,
    pub vec_of_structs: Vec<TestStruct2>,
}

#[derive(Debug)]
pub struct StructError(EnumError);
impl std::fmt::Display for StructError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StructError: {}", self.0)
    }
}
impl std::error::Error for StructError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(Debug)]
pub struct SimpleError;
impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SimpleError")
    }
}
impl std::error::Error for SimpleError {}

#[derive(Debug)]
pub enum EnumError {
    VariantOne,
    VariantTwo(SimpleError),
}
impl std::fmt::Display for EnumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnumError::VariantOne => write!(f, "EnumError: VariantOne"),
            EnumError::VariantTwo(_) => write!(f, "EnumError: VariantTwo"),
        }
    }
}
impl std::error::Error for EnumError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EnumError::VariantOne => None,
            EnumError::VariantTwo(ctx) => Some(ctx),
        }
    }
}

#[ffi]
pub fn function_with_primitive_result(error: bool) -> Result<i32, StructError> {
    if error {
        Err(StructError(EnumError::VariantTwo(SimpleError)))
    } else {
        Ok(123)
    }
}

#[ffi]
pub fn function_with_bool_result(error: bool) -> Result<bool, SimpleError> {
    if error { Err(SimpleError) } else { Ok(true) }
}

#[ffi]
pub fn function_with_string_result(error: bool) -> Result<String, EnumError> {
    if error {
        Err(EnumError::VariantOne)
    } else {
        Ok("No error".to_string())
    }
}

#[ffi]
pub fn function_with_struct_result(error: bool) -> Result<TestStruct2, StructError> {
    if error {
        Err(StructError(EnumError::VariantTwo(SimpleError)))
    } else {
        Ok(TestStruct2 { i32_field: 256 })
    }
}

#[ffi]
pub fn function_with_enum_result(error: bool) -> Result<TestStatus, EnumError> {
    if error {
        Err(EnumError::VariantOne)
    } else {
        Ok(TestStatus::Pending)
    }
}

#[ffi]
pub fn function_with_vec_int_result(error: bool) -> Result<Vec<i32>, StructError> {
    if error {
        Err(StructError(EnumError::VariantTwo(SimpleError)))
    } else {
        Ok(vec![10, 20, 30])
    }
}

#[ffi]
pub fn function_with_vec_bool_result(error: bool) -> Result<Vec<bool>, SimpleError> {
    if error {
        Err(SimpleError)
    } else {
        Ok(vec![true, false, false])
    }
}

#[ffi]
pub fn function_with_vec_string_result(error: bool) -> Result<Vec<String>, EnumError> {
    if error {
        Err(EnumError::VariantOne)
    } else {
        Ok(vec![
            "One".to_string(),
            "Two".to_string(),
            "Three".to_string(),
        ])
    }
}

#[ffi]
pub fn function_with_vec_struct_result(error: bool) -> Result<Vec<TestStruct2>, StructError> {
    if error {
        Err(StructError(EnumError::VariantTwo(SimpleError)))
    } else {
        Ok(vec![
            TestStruct2 { i32_field: 512 },
            TestStruct2 { i32_field: 1024 },
        ])
    }
}

#[ffi]
pub fn function_with_vec_enum_result(error: bool) -> Result<Vec<TestStatus>, EnumError> {
    if error {
        Err(EnumError::VariantOne)
    } else {
        Ok(vec![
            TestStatus::Pending,
            TestStatus::Active,
            TestStatus::Inactive,
        ])
    }
}
