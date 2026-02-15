use hi_ffi::ffi;

#[ffi]
pub fn take_status_before_it_is_defined(status: TestStatus) {
    assert_eq!(status, TestStatus::Pending);
}

#[ffi]
pub fn take_struct_and_return_status_before_they_are_defined(s: TestStruct) -> TestStatus {
    assert_eq!(s.i32_field, -5);
    TestStatus::Pending
}

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

    pub fn public_method_taking_primitives(&self, i: i32, b: bool) {
        assert_eq!(i, 10);
        assert!(!b);
    }

    pub fn public_method_taking_string(&self, s: String) {
        assert_eq!(s, "Test string from caller");
    }

    pub fn public_method_taking_struct(&self, s: TestStruct2) {
        assert_eq!(s, TestStruct2 { i32_field: 55 });
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
        if b { str1 } else { str2 }
    }

    // static method
    pub fn static_method() {}

    pub fn static_method_taking_primitives(i: i32, b: bool) {
        assert_eq!(i, 20);
        assert!(b);
    }
    pub fn static_method_taking_string(s: String) {
        assert_eq!(s, "static method string");
    }
    pub fn static_method_taking_struct(s: TestStruct2) {
        assert_eq!(s, TestStruct2 { i32_field: 77 });
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
        if b { str1 } else { str2 }
    }
    pub fn static_combo_struct_method(s1: TestStruct2, _s2: TestStruct2) -> TestStruct2 {
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

    pub fn static_method_taking_vec_of_primitives(vec: Vec<u16>) {
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

    pub fn method_with_int_result(&self, error: bool) -> Result<i64, StructError> {
        if error {
            Err(StructError(EnumError::VariantTwo(SimpleError)))
        } else {
            Ok(16)
        }
    }

    pub fn static_method_with_bool_result(error: bool) -> Result<bool, SimpleError> {
        if error { Err(SimpleError) } else { Ok(true) }
    }

    pub fn method_with_string_result(&self) -> Result<String, SimpleError> {
        Ok("Ok!".to_string())
    }

    pub fn static_method_with_struct_result() -> Result<TestStruct, SimpleError> {
        Ok(TestStruct {
            i32_field: 267,
            ..Default::default()
        })
    }

    pub fn method_with_enum_result(&self) -> Result<TestStatus, SimpleError> {
        Ok(TestStatus::Pending)
    }

    pub fn method_with_vec_of_ints_result(&self) -> Result<Vec<i8>, SimpleError> {
        Ok(vec![1, 2, 7])
    }

    pub fn static_method_with_vec_of_bools_result() -> Result<Vec<bool>, SimpleError> {
        Ok(vec![true, false, false])
    }

    pub fn method_with_vec_of_strings_result(&self) -> Result<Vec<String>, SimpleError> {
        Ok(vec!["some".to_string(), "string".to_string()])
    }

    pub fn static_method_with_vec_of_structs_result() -> Result<Vec<TestStruct2>, SimpleError> {
        let s1 = TestStruct2 { i32_field: 2 };
        let s2 = TestStruct2 { i32_field: -5 };
        Ok(vec![s1, s2])
    }

    pub fn method_with_vec_of_bools_result(&self) -> Result<Vec<bool>, SimpleError> {
        Ok(vec![false, true, true, true])
    }

    pub fn static_method_with_vec_of_enum_result() -> Result<Vec<TestStatus>, SimpleError> {
        Ok(vec![
            TestStatus::Inactive,
            TestStatus::Pending,
            TestStatus::Active,
        ])
    }

    pub fn method_taking_opt_int(&self, i: Option<i32>, assert_some: bool) {
        if assert_some {
            assert_eq!(Some(20), i);
        } else {
            assert!(i.is_none());
        }
    }

    pub fn static_method_taking_opt_string(s: Option<String>, assert_some: bool) {
        if assert_some {
            assert_eq!(Some("Optional string".to_string()), s);
        } else {
            assert!(s.is_none());
        }
    }

    pub fn method_taking_opt_bool(&self, b: Option<bool>, assert_some: bool) {
        if assert_some {
            assert_eq!(Some(false), b);
        } else {
            assert!(b.is_none());
        }
    }

    pub fn static_method_taking_opt_enum(e: Option<TestStatus>, assert_some: bool) {
        if assert_some {
            assert_eq!(Some(TestStatus::Active), e);
        } else {
            assert!(e.is_none());
        }
    }

    pub fn method_taking_opt_struct(&self, s: Option<TestStruct2>, assert_some: bool) {
        if assert_some {
            assert_eq!(Some(TestStruct2 { i32_field: 789 }), s);
        } else {
            assert!(s.is_none());
        }
    }

    pub fn method_returning_opt_int(&self, some: bool) -> Option<i32> {
        if some { Some(30) } else { None }
    }

    pub fn static_method_returning_opt_bool(some: bool) -> Option<bool> {
        if some { Some(false) } else { None }
    }

    pub fn method_returning_opt_string(&self, some: bool) -> Option<String> {
        if some {
            Some("Optional string from Rust".to_string())
        } else {
            None
        }
    }

    pub fn static_method_returning_opt_enum(some: bool) -> Option<TestStatus> {
        if some {
            Some(TestStatus::Inactive)
        } else {
            None
        }
    }

    pub fn method_returning_opt_struct(&self, some: bool) -> Option<TestStruct2> {
        if some {
            Some(TestStruct2 { i32_field: 654 })
        } else {
            None
        }
    }
}

#[ffi]
#[derive(Default, Clone, Serialize, Debug, PartialEq)]
pub struct TestStruct2 {
    pub i32_field: i32,
}

#[ffi]
fn simple_function() {}

#[ffi]
fn function_with_primitive_args(i: i32, b: bool) {
    assert_eq!(i, 100);
    assert!(b);
}

#[ffi]
fn function_with_string_arg(s: String) {
    assert_eq!(s, "Hello, World!");
}

#[ffi]
fn function_with_primitive_and_string_arg(i: i32, b: bool, s: String) {
    assert_eq!(i, 42);
    assert!(!b);
    assert_eq!(s, "Complex function!");
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
fn combo_function(str1: String, str2: String, b: bool, s: TestStruct) -> String {
    assert_eq!(s.i32_field, 0);
    if b { str1 } else { str2 }
}

#[ffi]
fn function_taking_struct(s: TestStruct2) {
    assert_eq!(s, TestStruct2 { i32_field: 55 });
}

#[ffi]
fn function_returning_struct() -> TestStruct2 {
    TestStruct2 { i32_field: 48 }
}

#[ffi]
fn combo_struct_function(s1: TestStruct, _s2: TestStruct, _s3: TestStruct2) -> TestStruct {
    s1
}

#[ffi]
fn function_taking_vec_of_primitives(vec: Vec<i32>) {
    assert_eq!(vec![1, 2, 3, 4, 5], vec);
}

#[ffi]
fn function_taking_vec_of_bools(vec: Vec<bool>) {
    assert_eq!(vec![true, false, true, true], vec);
}

#[ffi]
fn function_taking_vec_of_strings(vec: Vec<String>) {
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
    let s1 = TestStruct {
        i32_field: 15,
        ..Default::default()
    };
    let s2 = TestStruct {
        i32_field: 17,
        ..Default::default()
    };
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
pub fn function_with_unit_expression_result() -> Result<(), SimpleError> {
    Ok(())
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

// Optionals

#[ffi]
pub fn function_taking_some_int(i: Option<i32>, assert_some: bool) {
    if assert_some {
        assert_eq!(Some(10), i);
    } else {
        assert!(i.is_none());
    }
}

#[ffi]
pub fn function_taking_some_bool(b: Option<bool>, assert_some: bool) {
    if assert_some {
        assert_eq!(Some(true), b);
    } else {
        assert!(b.is_none());
    }
}

#[ffi]
pub fn function_taking_some_string(s: Option<String>, assert_some: bool) {
    if assert_some {
        assert_eq!(Some("Some string".to_string()), s);
    } else {
        assert!(s.is_none());
    }
}

#[ffi]
pub fn function_taking_some_enum(e: Option<TestStatus>, assert_some: bool) {
    if assert_some {
        assert_eq!(Some(TestStatus::Pending), e);
    } else {
        assert!(e.is_none());
    }
}

#[ffi]
pub(crate) fn function_taking_some_struct(e: Option<TestStruct>, assert_some: bool) {
    if assert_some {
        assert_eq!(
            Some(TestStruct {
                i32_field: 567,
                ..Default::default()
            }),
            e
        );
    } else {
        assert!(e.is_none());
    }
}

#[ffi]
pub(crate) fn function_returning_opt_int(some: bool) -> Option<i32> {
    if some { Some(100) } else { None }
}
#[ffi]
pub(crate) fn function_returning_opt_bool(some: bool) -> Option<bool> {
    if some { Some(true) } else { None }
}
#[ffi]
pub(crate) fn function_returning_opt_string(some: bool) -> Option<String> {
    if some {
        Some("Some Rust String".to_string())
    } else {
        None
    }
}
#[ffi]
pub(crate) fn function_returning_opt_enum(some: bool) -> Option<TestStatus> {
    if some {
        Some(TestStatus::Pending)
    } else {
        None
    }
}
#[ffi]
pub(crate) fn function_returning_opt_struct(some: bool) -> Option<TestStruct2> {
    if some {
        Some(TestStruct2 { i32_field: 234 })
    } else {
        None
    }
}
