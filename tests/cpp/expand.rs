#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use hi_ffi::ffi;

use serde::Serialize;

struct TestStruct {
    // generate getter and setter
    // check if other attributes are preserved
    #[serde(skip_serializing)]
    i32_field: i32,

    // generate getter only
    bool_field: bool,

    // generate getter and setter
    pub string_field: String,

    // skip setter
    _no_setter: i32,

    pub f32_field: f32,

    // don't generate getter and setter
    _skip_field: i32,

    struct_field: TestStruct2,
}
#[automatically_derived]
impl ::core::default::Default for TestStruct {
    #[inline]
    fn default() -> TestStruct {
        TestStruct {
            i32_field: ::core::default::Default::default(),
            bool_field: ::core::default::Default::default(),
            string_field: ::core::default::Default::default(),
            _no_setter: ::core::default::Default::default(),
            f32_field: ::core::default::Default::default(),
            _skip_field: ::core::default::Default::default(),
            struct_field: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for TestStruct {
    #[inline]
    fn clone(&self) -> TestStruct {
        TestStruct {
            i32_field: ::core::clone::Clone::clone(&self.i32_field),
            bool_field: ::core::clone::Clone::clone(&self.bool_field),
            string_field: ::core::clone::Clone::clone(&self.string_field),
            _no_setter: ::core::clone::Clone::clone(&self._no_setter),
            f32_field: ::core::clone::Clone::clone(&self.f32_field),
            _skip_field: ::core::clone::Clone::clone(&self._skip_field),
            struct_field: ::core::clone::Clone::clone(&self.struct_field),
        }
    }
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for TestStruct {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "TestStruct",
                false as usize + 1 + 1 + 1 + 1 + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "bool_field",
                &self.bool_field,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "string_field",
                &self.string_field,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "_no_setter",
                &self._no_setter,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "f32_field",
                &self.f32_field,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "_skip_field",
                &self._skip_field,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "struct_field",
                &self.struct_field,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for TestStruct {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "i32_field",
            "bool_field",
            "string_field",
            "_no_setter",
            "f32_field",
            "_skip_field",
            "struct_field",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.i32_field,
            &self.bool_field,
            &self.string_field,
            &self._no_setter,
            &self.f32_field,
            &self._skip_field,
            &&self.struct_field,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(f, "TestStruct", names, values)
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for TestStruct {}
#[automatically_derived]
impl ::core::cmp::PartialEq for TestStruct {
    #[inline]
    fn eq(&self, other: &TestStruct) -> bool {
        self.i32_field == other.i32_field
            && self.bool_field == other.bool_field
            && self._no_setter == other._no_setter
            && self.f32_field == other.f32_field
            && self._skip_field == other._skip_field
            && self.string_field == other.string_field
            && self.struct_field == other.struct_field
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__get_i32_field")]
pub unsafe extern "C" fn TestStruct_get_i32_field(_self: *mut TestStruct) -> i32 {
    unsafe { (&*_self).i32_field }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__set_i32_field")]
pub unsafe extern "C" fn TestStruct_set_i32_field(_self: *mut TestStruct, value: i32) {
    unsafe {
        (&mut *_self).i32_field = value;
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__get_bool_field")]
pub unsafe extern "C" fn TestStruct_get_bool_field(_self: *mut TestStruct) -> bool {
    unsafe { (&*_self).bool_field }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__set_bool_field")]
pub unsafe extern "C" fn TestStruct_set_bool_field(_self: *mut TestStruct, value: bool) {
    unsafe {
        (&mut *_self).bool_field = value;
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__get_string_field")]
pub unsafe extern "C" fn TestStruct_get_string_field(_self: *mut TestStruct) -> *mut FfiSlice {
    unsafe {
        Box::into_raw(Box::new(FfiSlice {
            ptr: (&*_self).string_field.as_ptr(),
            len: (&*_self).string_field.len(),
        }))
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__set_string_field")]
pub unsafe extern "C" fn TestStruct_set_string_field(
    _self: *mut TestStruct,
    ptr: *const i8,
    _len: usize,
) {
    unsafe {
        let s = std::ffi::CStr::from_ptr(ptr).to_str().unwrap().to_owned();
        (&mut *_self).string_field = s;
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__get__no_setter")]
pub unsafe extern "C" fn TestStruct_get__no_setter(_self: *mut TestStruct) -> i32 {
    unsafe { (&*_self)._no_setter }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__get_f32_field")]
pub unsafe extern "C" fn TestStruct_get_f32_field(_self: *mut TestStruct) -> f32 {
    unsafe { (&*_self).f32_field }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__set_f32_field")]
pub unsafe extern "C" fn TestStruct_set_f32_field(_self: *mut TestStruct, value: f32) {
    unsafe {
        (&mut *_self).f32_field = value;
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__get_struct_field")]
pub unsafe extern "C" fn TestStruct_get_struct_field(_self: *mut TestStruct) -> *mut TestStruct2 {
    unsafe { Box::leak(Box::new((&*_self).struct_field.clone())) }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__set_struct_field")]
pub unsafe extern "C" fn TestStruct_set_struct_field(
    _self: *mut TestStruct,
    value: *mut TestStruct2,
) {
    unsafe {
        (&mut *_self).struct_field = (*value).clone();
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__default")]
pub unsafe extern "C" fn TestStruct__default() -> *mut TestStruct {
    unsafe {
        let instance = Box::new(TestStruct::default());
        let ptr = Box::into_raw(instance);
        ptr
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____TestStruct__drop")]
pub unsafe extern "C" fn TestStruct_drop(_self: *mut TestStruct) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct__clone")]
pub unsafe extern "C" fn TestStruct_clone(_self: *mut TestStruct) -> *mut TestStruct {
    unsafe {
        let cloned: Box<TestStruct> = Box::new((*_self).clone());
        Box::into_raw(cloned)
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__rust_ref_dyn_err_drop")]
pub unsafe extern "C" fn rust_ref_dyn_err_drop(_self: *const &dyn std::error::Error) {
    unsafe {
        let _ = Box::from_raw(_self as *mut &dyn std::error::Error);
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__rust_ref_dyn_err_desc")]
pub unsafe extern "C" fn rust_ref_dyn_err_desc(
    _self: *const &dyn std::error::Error,
) -> *const String {
    unsafe {
        let err = &*(_self);
        Box::into_raw(Box::new(err.to_string())) as _
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__rust_ref_dyn_err_source")]
pub unsafe extern "C" fn rust_ref_dyn_err_source(
    _self: *const &dyn std::error::Error,
) -> *const std::ffi::c_void {
    unsafe {
        let err = &*(_self);
        match err.source() {
            Some(source_err) => {
                Box::into_raw(Box::new(source_err.clone())) as *const std::ffi::c_void
            }
            None => std::ptr::null(),
        }
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__rust_arc_dyn_err_clone")]
pub unsafe extern "C" fn rust_arc_dyn_err_clone(
    _self: *const std::sync::Arc<dyn std::error::Error>,
) -> *mut std::sync::Arc<dyn std::error::Error> {
    unsafe {
        let arc_err = &*(_self);
        let arc_err_clone = std::sync::Arc::clone(arc_err);
        Box::into_raw(Box::new(arc_err_clone))
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__rust_arc_dyn_err_drop")]
pub unsafe extern "C" fn rust_arc_dyn_err_drop(_self: *mut std::sync::Arc<dyn std::error::Error>) {
    unsafe {
        let _ = Box::from_raw(_self);
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__rust_arc_dyn_err_desc")]
pub unsafe extern "C" fn rust_arc_dyn_err_desc(
    _self: *const std::sync::Arc<dyn std::error::Error>,
) -> *const String {
    unsafe {
        let err = &*(_self);
        Box::into_raw(Box::new(err.to_string())) as _
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__rust_arc_dyn_err_source")]
pub unsafe extern "C" fn rust_arc_dyn_err_source(
    _self: *const std::sync::Arc<dyn std::error::Error>,
) -> *const std::ffi::c_void {
    unsafe {
        let err = &*(_self);
        match err.source() {
            Some(source_err) => {
                Box::into_raw(Box::new(source_err.clone())) as *const std::ffi::c_void
            }
            None => std::ptr::null(),
        }
    }
}
#[repr(C)]
pub struct FfiSlice {
    pub ptr: *const u8,
    pub len: usize,
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__slice_len")]
pub unsafe extern "C" fn slice_len(_self: *const FfiSlice) -> usize {
    unsafe { (*_self).len }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__slice_ptr")]
pub unsafe extern "C" fn slice_ptr(_self: *const FfiSlice) -> *const std::os::raw::c_char {
    unsafe { (*_self).ptr as *const std::os::raw::c_char }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__slice_drop")]
pub unsafe extern "C" fn slice_drop(_self: *mut FfiSlice) {
    unsafe {
        let _ = Box::from_raw(_self);
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__rust_string_drop")]
pub unsafe extern "C" fn rust_string_drop(_self: *mut std::ffi::c_void) {
    unsafe {
        let _ = Box::from_raw(_self as *mut String);
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__rust_string_data")]
pub unsafe extern "C" fn rust_string_data(_self: *const std::ffi::c_void) -> *const u8 {
    unsafe {
        let s = &*(_self as *const String);
        s.as_ptr()
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi__rust_string_len")]
pub unsafe extern "C" fn rust_string_len(_self: *const std::ffi::c_void) -> usize {
    unsafe {
        let s = &*(_self as *const String);
        s.len()
    }
}

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
        if b {
            str1
        } else {
            str2
        }
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
        if b {
            str1
        } else {
            str2
        }
    }
    pub fn static_combo_struct_method(s1: TestStruct2, _s2: TestStruct2) -> TestStruct2 {
        // println!("Rust: Static combo struct method: s1 = {s1:?}, s2 = {_s2:?}");
        s1
    }

    pub fn public_method_taking_vec_of_primitives(&self, vec: Vec<i32>) {
        // println!("Rust: Simple function called"); // This line causes still reachable resources in valgrind report

        // println!("Rust: Function with args called: i = {_i}, s = {_b}"); // This line causes still reachable resources in valgrind report

        // println!("Rust: Function with string arg called: s = {_s}"); // This line causes still reachable resources in valgrind report

        // println!("Rust: Function with args called: i = {_i}, s = {_b}, s = {_s}"); // This line causes still reachable resources in valgrind report

        // println!("{str1} {str2} {b} {_s:?}");

        // println!("Rust: Function with struct arg called: s = {_s:?}");

        // println!("Rust: Combo struct function called: s1 = {s1:?}, s2 = {_s2:?}, s3 = {_s3:?}");

        // println!(
        //     "Rust: Function with vector of primitives called: {:?}",
        //     vec
        // );

        // println!("Rust: Function with vector of bools called: {:?}", _vec);

        // println!("Rust: Function with vector of strings called: {:?}", _vec);

        // println!("Rust: Function with vector of structs called: {:?}", vec);

        // Having Drop defined causes still reachable resources in valgrind report
        // println!("Dropping TestStruct");

        // println!("Dropping TestStruct2");

        match (
            &<[_]>::into_vec(::alloc::boxed::box_new([1, 2, 3, 4, 5])),
            &vec,
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    pub fn public_method_taking_vec_of_strings(&self, vec: Vec<String>) {
        match (
            &<[_]>::into_vec(::alloc::boxed::box_new([
                "Hello".to_string(),
                "World".to_string(),
            ])),
            &vec,
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    pub fn public_method_taking_vec_of_structs(&self, vec: Vec<TestStruct2>) {
        let expected = <[_]>::into_vec(::alloc::boxed::box_new([
            TestStruct2 { i32_field: 42 },
            TestStruct2 { i32_field: 24 },
        ]));
        match (&expected, &vec) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    pub fn static_method_taking_vec_of_primitives(vec: Vec<i32>) {
        match (
            &<[_]>::into_vec(::alloc::boxed::box_new([6, 7, 8, 9, 10])),
            &vec,
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    pub fn static_method_taking_vec_of_strings(vec: Vec<String>) {
        match (
            &<[_]>::into_vec(::alloc::boxed::box_new([
                "Static".to_string(),
                "Method".to_string(),
            ])),
            &vec,
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    pub fn static_method_taking_vec_of_structs(vec: Vec<TestStruct2>) {
        let expected = <[_]>::into_vec(::alloc::boxed::box_new([
            TestStruct2 { i32_field: 100 },
            TestStruct2 { i32_field: 200 },
        ]));
        match (&expected, &vec) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    pub fn public_method_returning_vec_of_primitives(&self) -> Vec<i32> {
        <[_]>::into_vec(::alloc::boxed::box_new([10, 20, 30, 40, 50]))
    }
    pub fn public_method_returning_vec_of_strings(&self) -> Vec<String> {
        <[_]>::into_vec(::alloc::boxed::box_new([
            "Method".to_string(),
            "Vector".to_string(),
            "Return".to_string(),
        ]))
    }
    pub fn public_method_returning_vec_of_structs(&self) -> Vec<TestStruct2> {
        <[_]>::into_vec(::alloc::boxed::box_new([
            TestStruct2 { i32_field: 300 },
            TestStruct2 { i32_field: 400 },
        ]))
    }
    pub fn static_method_returning_vec_of_primitives() -> Vec<i32> {
        <[_]>::into_vec(::alloc::boxed::box_new([60, 70, 80, 90, 100]))
    }
    pub fn static_method_returning_vec_of_strings() -> Vec<String> {
        <[_]>::into_vec(::alloc::boxed::box_new([
            "Static".to_string(),
            "Method".to_string(),
            "Vector".to_string(),
        ]))
    }
    pub fn static_method_returning_vec_of_structs() -> Vec<TestStruct2> {
        <[_]>::into_vec(::alloc::boxed::box_new([
            TestStruct2 { i32_field: 500 },
            TestStruct2 { i32_field: 600 },
        ]))
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method(_self: *mut TestStruct) {
    let result = (&mut *_self).public_method();
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_taking_primitives")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_taking_primitives(
    _self: *mut TestStruct,
    _i: i32,
    _b: bool,
) {
    let result = (&mut *_self).public_method_taking_primitives(_i, _b);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_taking_string")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_taking_string(
    _self: *mut TestStruct,
    _s: *const i8,
) {
    let _s = unsafe { std::ffi::CStr::from_ptr(_s).to_str().unwrap().to_owned() };
    let result = (&mut *_self).public_method_taking_string(_s);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_taking_struct")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_taking_struct(
    _self: *mut TestStruct,
    _s: *mut TestStruct2,
) {
    let _s = unsafe { (*_s).clone() };
    let result = (&mut *_self).public_method_taking_struct(_s);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_returning_primitive")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_returning_primitive(
    _self: *mut TestStruct,
) -> i32 {
    let result = (&mut *_self).public_method_returning_primitive();
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_returning_bool")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_returning_bool(
    _self: *mut TestStruct,
) -> bool {
    let result = (&mut *_self).public_method_returning_bool();
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_returning_string")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_returning_string(
    _self: *mut TestStruct,
) -> *mut String {
    let result = (&mut *_self).public_method_returning_string();
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_returning_struct")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_returning_struct(
    _self: *mut TestStruct,
) -> *mut TestStruct2 {
    let result = (&mut *_self).public_method_returning_struct();
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_combo_method")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_combo_method(
    _self: *mut TestStruct,
    str1: *const i8,
    str2: *const i8,
    b: bool,
) -> *mut String {
    let str1 = unsafe { std::ffi::CStr::from_ptr(str1).to_str().unwrap().to_owned() };
    let str2 = unsafe { std::ffi::CStr::from_ptr(str2).to_str().unwrap().to_owned() };
    let result = (&mut *_self).combo_method(str1, str2, b);
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method() {
    let result = TestStruct::static_method();
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_taking_primitives")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_taking_primitives(_i: i32, _b: bool) {
    let result = TestStruct::static_method_taking_primitives(_i, _b);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_taking_string")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_taking_string(_s: *const i8) {
    let _s = unsafe { std::ffi::CStr::from_ptr(_s).to_str().unwrap().to_owned() };
    let result = TestStruct::static_method_taking_string(_s);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_taking_struct")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_taking_struct(_s: *mut TestStruct2) {
    let _s = unsafe { (*_s).clone() };
    let result = TestStruct::static_method_taking_struct(_s);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_returning_primitive")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_returning_primitive() -> i32 {
    let result = TestStruct::static_method_returning_primitive();
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_returning_string")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_returning_string() -> *mut String {
    let result = TestStruct::static_method_returning_string();
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_returning_struct")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_returning_struct() -> *mut TestStruct2
{
    let result = TestStruct::static_method_returning_struct();
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_combo_method")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_combo_method(
    str1: *const i8,
    str2: *const i8,
    b: bool,
) -> *mut String {
    let str1 = unsafe { std::ffi::CStr::from_ptr(str1).to_str().unwrap().to_owned() };
    let str2 = unsafe { std::ffi::CStr::from_ptr(str2).to_str().unwrap().to_owned() };
    let result = TestStruct::static_combo_method(str1, str2, b);
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_combo_struct_method")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_combo_struct_method(
    s1: *mut TestStruct2,
    _s2: *mut TestStruct2,
) -> *mut TestStruct2 {
    let s1 = unsafe { (*s1).clone() };
    let _s2 = unsafe { (*_s2).clone() };
    let result = TestStruct::static_combo_struct_method(s1, _s2);
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_taking_vec_of_primitives")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_taking_vec_of_primitives(
    _self: *mut TestStruct,
    vec: *mut Vec<i32>,
) {
    let mut new_vec = Vec::new();
    std::mem::swap(&mut new_vec, unsafe { &mut (*vec) });
    let result = (&mut *_self).public_method_taking_vec_of_primitives(new_vec);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_taking_vec_of_strings")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_taking_vec_of_strings(
    _self: *mut TestStruct,
    vec: *mut Vec<String>,
) {
    let mut new_vec = Vec::new();
    std::mem::swap(&mut new_vec, unsafe { &mut (*vec) });
    let result = (&mut *_self).public_method_taking_vec_of_strings(new_vec);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_taking_vec_of_structs")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_taking_vec_of_structs(
    _self: *mut TestStruct,
    vec: *mut Vec<TestStruct2>,
) {
    let mut new_vec = Vec::new();
    std::mem::swap(&mut new_vec, unsafe { &mut (*vec) });
    let result = (&mut *_self).public_method_taking_vec_of_structs(new_vec);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_taking_vec_of_primitives")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_taking_vec_of_primitives(
    vec: *mut Vec<i32>,
) {
    let mut new_vec = Vec::new();
    std::mem::swap(&mut new_vec, unsafe { &mut (*vec) });
    let result = TestStruct::static_method_taking_vec_of_primitives(new_vec);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_taking_vec_of_strings")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_taking_vec_of_strings(
    vec: *mut Vec<String>,
) {
    let mut new_vec = Vec::new();
    std::mem::swap(&mut new_vec, unsafe { &mut (*vec) });
    let result = TestStruct::static_method_taking_vec_of_strings(new_vec);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_taking_vec_of_structs")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_taking_vec_of_structs(
    vec: *mut Vec<TestStruct2>,
) {
    let mut new_vec = Vec::new();
    std::mem::swap(&mut new_vec, unsafe { &mut (*vec) });
    let result = TestStruct::static_method_taking_vec_of_structs(new_vec);
    result
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_returning_vec_of_primitives")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_returning_vec_of_primitives(
    _self: *mut TestStruct,
) -> *mut Vec<i32> {
    let result = (&mut *_self).public_method_returning_vec_of_primitives();
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_returning_vec_of_strings")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_returning_vec_of_strings(
    _self: *mut TestStruct,
) -> *mut Vec<String> {
    let result = (&mut *_self).public_method_returning_vec_of_strings();
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_public_method_returning_vec_of_structs")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_public_method_returning_vec_of_structs(
    _self: *mut TestStruct,
) -> *mut Vec<TestStruct2> {
    let result = (&mut *_self).public_method_returning_vec_of_structs();
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_returning_vec_of_primitives")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_returning_vec_of_primitives(
) -> *mut Vec<i32> {
    let result = TestStruct::static_method_returning_vec_of_primitives();
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_returning_vec_of_strings")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_returning_vec_of_strings(
) -> *mut Vec<String> {
    let result = TestStruct::static_method_returning_vec_of_strings();
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi___TestStruct_static_method_returning_vec_of_structs")]
pub unsafe extern "C" fn ffi_wrapper_teststruct_static_method_returning_vec_of_structs(
) -> *mut Vec<TestStruct2> {
    let result = TestStruct::static_method_returning_vec_of_structs();
    Box::into_raw(Box::new(result))
}
pub struct TestStruct2 {
    pub i32_field: i32,
}
#[automatically_derived]
impl ::core::default::Default for TestStruct2 {
    #[inline]
    fn default() -> TestStruct2 {
        TestStruct2 {
            i32_field: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for TestStruct2 {
    #[inline]
    fn clone(&self) -> TestStruct2 {
        TestStruct2 {
            i32_field: ::core::clone::Clone::clone(&self.i32_field),
        }
    }
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for TestStruct2 {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "TestStruct2",
                false as usize + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "i32_field",
                &self.i32_field,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for TestStruct2 {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "TestStruct2",
            "i32_field",
            &&self.i32_field,
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for TestStruct2 {}
#[automatically_derived]
impl ::core::cmp::PartialEq for TestStruct2 {
    #[inline]
    fn eq(&self, other: &TestStruct2) -> bool {
        self.i32_field == other.i32_field
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct2__get_i32_field")]
pub unsafe extern "C" fn TestStruct2_get_i32_field(_self: *mut TestStruct2) -> i32 {
    unsafe { (&*_self).i32_field }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct2__set_i32_field")]
pub unsafe extern "C" fn TestStruct2_set_i32_field(_self: *mut TestStruct2, value: i32) {
    unsafe {
        (&mut *_self).i32_field = value;
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct2__default")]
pub unsafe extern "C" fn TestStruct2__default() -> *mut TestStruct2 {
    unsafe {
        let instance = Box::new(TestStruct2::default());
        let ptr = Box::into_raw(instance);
        ptr
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____TestStruct2__drop")]
pub unsafe extern "C" fn TestStruct2_drop(_self: *mut TestStruct2) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____TestStruct2__clone")]
pub unsafe extern "C" fn TestStruct2_clone(_self: *mut TestStruct2) -> *mut TestStruct2 {
    unsafe {
        let cloned: Box<TestStruct2> = Box::new((*_self).clone());
        Box::into_raw(cloned)
    }
}
fn simple_function() {}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___simple_function")]
pub unsafe extern "C" fn ffi_wrapper_simple_function() {
    let result = simple_function();
    result
}
fn function_with_primitive_args(_i: i32, _b: bool) {}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_with_primitive_args")]
pub unsafe extern "C" fn ffi_wrapper_function_with_primitive_args(_i: i32, _b: bool) {
    let result = function_with_primitive_args(_i, _b);
    result
}
fn function_with_string_arg(_s: String) {}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_with_string_arg")]
pub unsafe extern "C" fn ffi_wrapper_function_with_string_arg(_s: *const i8) {
    let _s = unsafe { std::ffi::CStr::from_ptr(_s).to_str().unwrap().to_owned() };
    let result = function_with_string_arg(_s);
    result
}
fn function_with_primitive_and_string_arg(_i: i32, _b: bool, _s: String) {}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_with_primitive_and_string_arg")]
pub unsafe extern "C" fn ffi_wrapper_function_with_primitive_and_string_arg(
    _i: i32,
    _b: bool,
    _s: *const i8,
) {
    let _s = unsafe { std::ffi::CStr::from_ptr(_s).to_str().unwrap().to_owned() };
    let result = function_with_primitive_and_string_arg(_i, _b, _s);
    result
}
fn function_return_primitive() -> i32 {
    42
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_return_primitive")]
pub unsafe extern "C" fn ffi_wrapper_function_return_primitive() -> i32 {
    let result = function_return_primitive();
    result
}
fn function_return_float() -> f64 {
    5.21
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_return_float")]
pub unsafe extern "C" fn ffi_wrapper_function_return_float() -> f64 {
    let result = function_return_float();
    result
}
fn function_return_negated_bool(input: bool) -> bool {
    !input
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_return_negated_bool")]
pub unsafe extern "C" fn ffi_wrapper_function_return_negated_bool(input: bool) -> bool {
    let result = function_return_negated_bool(input);
    result
}
fn function_return_string() -> String {
    "String returned from Rust".to_string()
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_return_string")]
pub unsafe extern "C" fn ffi_wrapper_function_return_string() -> *mut String {
    let result = function_return_string();
    Box::into_raw(Box::new(result))
}
fn combo_function(str1: String, str2: String, b: bool, _s: TestStruct) -> String {
    if b {
        str1
    } else {
        str2
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___combo_function")]
pub unsafe extern "C" fn ffi_wrapper_combo_function(
    str1: *const i8,
    str2: *const i8,
    b: bool,
    _s: *mut TestStruct,
) -> *mut String {
    let str1 = unsafe { std::ffi::CStr::from_ptr(str1).to_str().unwrap().to_owned() };
    let str2 = unsafe { std::ffi::CStr::from_ptr(str2).to_str().unwrap().to_owned() };
    let _s = unsafe { (*_s).clone() };
    let result = combo_function(str1, str2, b, _s);
    Box::into_raw(Box::new(result))
}
fn function_taking_struct(_s: TestStruct2) {}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_taking_struct")]
pub unsafe extern "C" fn ffi_wrapper_function_taking_struct(_s: *mut TestStruct2) {
    let _s = unsafe { (*_s).clone() };
    let result = function_taking_struct(_s);
    result
}
fn function_returning_struct() -> TestStruct2 {
    TestStruct2 { i32_field: 48 }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_returning_struct")]
pub unsafe extern "C" fn ffi_wrapper_function_returning_struct() -> *mut TestStruct2 {
    let result = function_returning_struct();
    Box::into_raw(Box::new(result))
}
fn combo_struct_function(s1: TestStruct, _s2: TestStruct, _s3: TestStruct2) -> TestStruct {
    s1
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___combo_struct_function")]
pub unsafe extern "C" fn ffi_wrapper_combo_struct_function(
    s1: *mut TestStruct,
    _s2: *mut TestStruct,
    _s3: *mut TestStruct2,
) -> *mut TestStruct {
    let s1 = unsafe { (*s1).clone() };
    let _s2 = unsafe { (*_s2).clone() };
    let _s3 = unsafe { (*_s3).clone() };
    let result = combo_struct_function(s1, _s2, _s3);
    Box::into_raw(Box::new(result))
}
fn function_taking_vec_of_primitives(vec: Vec<i32>) {
    match (
        &<[_]>::into_vec(::alloc::boxed::box_new([1, 2, 3, 4, 5])),
        &vec,
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_taking_vec_of_primitives")]
pub unsafe extern "C" fn ffi_wrapper_function_taking_vec_of_primitives(vec: *mut Vec<i32>) {
    let mut new_vec = Vec::new();
    std::mem::swap(&mut new_vec, unsafe { &mut (*vec) });
    let result = function_taking_vec_of_primitives(new_vec);
    result
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_i32_vec")]
pub unsafe extern "C" fn drop_i32_vec(_self: *mut Vec<i32>) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____with_capacity_i32_vec")]
pub unsafe extern "C" fn with_capacity_i32_vec(capacity: usize) -> *mut Vec<i32> {
    unsafe {
        let vec = Box::new(Vec::with_capacity(capacity));
        Box::into_raw(vec)
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____push_i32_vec")]
pub unsafe extern "C" fn push_i32_vec(_self: *mut Vec<i32>, value: i32) {
    (&mut *_self).push(value);
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____len_i32_vec")]
pub unsafe extern "C" fn len_i32_vec(_self: *mut Vec<i32>) -> usize {
    (&*_self).len()
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____get_i32_vec")]
pub unsafe extern "C" fn get_i32_vec(_self: *mut Vec<i32>, index: usize) -> i32 {
    (&*_self)[index]
}
fn function_taking_vec_of_bools(vec: Vec<bool>) {
    match (
        &<[_]>::into_vec(::alloc::boxed::box_new([true, false, true, true])),
        &vec,
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_taking_vec_of_bools")]
pub unsafe extern "C" fn ffi_wrapper_function_taking_vec_of_bools(vec: *mut Vec<bool>) {
    let mut new_vec = Vec::new();
    std::mem::swap(&mut new_vec, unsafe { &mut (*vec) });
    let result = function_taking_vec_of_bools(new_vec);
    result
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_bool_vec")]
pub unsafe extern "C" fn drop_bool_vec(_self: *mut Vec<bool>) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____with_capacity_bool_vec")]
pub unsafe extern "C" fn with_capacity_bool_vec(capacity: usize) -> *mut Vec<bool> {
    unsafe {
        let vec = Box::new(Vec::with_capacity(capacity));
        Box::into_raw(vec)
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____push_bool_vec")]
pub unsafe extern "C" fn push_bool_vec(_self: *mut Vec<bool>, value: bool) {
    (&mut *_self).push(value);
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____len_bool_vec")]
pub unsafe extern "C" fn len_bool_vec(_self: *mut Vec<bool>) -> usize {
    (&*_self).len()
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____get_bool_vec")]
pub unsafe extern "C" fn get_bool_vec(_self: *mut Vec<bool>, index: usize) -> bool {
    (&*_self)[index]
}
fn function_taking_vec_of_strings(vec: Vec<String>) {
    match (
        &<[_]>::into_vec(::alloc::boxed::box_new([
            "Hello, Rust!",
            "Hello, C++!",
            "Hello, Python!",
            "Hello, Swift!",
        ])),
        &vec,
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_taking_vec_of_strings")]
pub unsafe extern "C" fn ffi_wrapper_function_taking_vec_of_strings(vec: *mut Vec<String>) {
    let mut new_vec = Vec::new();
    std::mem::swap(&mut new_vec, unsafe { &mut (*vec) });
    let result = function_taking_vec_of_strings(new_vec);
    result
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_String_vec")]
pub unsafe extern "C" fn drop_String_vec(_self: *mut Vec<String>) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____with_capacity_String_vec")]
pub unsafe extern "C" fn with_capacity_String_vec(capacity: usize) -> *mut Vec<String> {
    unsafe {
        let vec = Box::new(Vec::with_capacity(capacity));
        Box::into_raw(vec)
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____push_String_vec")]
pub unsafe extern "C" fn push_String_vec(_self: *mut Vec<String>, ptr: *const i8, _len: usize) {
    let value = std::ffi::CStr::from_ptr(ptr).to_str().unwrap().to_owned();
    (&mut *_self).push(value);
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____len_String_vec")]
pub unsafe extern "C" fn len_String_vec(_self: *mut Vec<String>) -> usize {
    (&*_self).len()
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____get_String_vec")]
pub unsafe extern "C" fn get_String_vec(
    _self: *mut Vec<String>,
    index: usize,
) -> *mut std::string::String {
    {
        let v = (&*_self)[index].clone();
        Box::into_raw(Box::new(v))
    }
}
fn function_taking_vec_of_structs(vec: Vec<TestStruct>) {
    let mut s1 = TestStruct::default();
    s1.i32_field = 15;
    let mut s2 = TestStruct::default();
    s2.i32_field = 17;
    let expected = <[_]>::into_vec(::alloc::boxed::box_new([s1, s2]));
    match (&expected, &vec) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_taking_vec_of_structs")]
pub unsafe extern "C" fn ffi_wrapper_function_taking_vec_of_structs(vec: *mut Vec<TestStruct>) {
    let mut new_vec = Vec::new();
    std::mem::swap(&mut new_vec, unsafe { &mut (*vec) });
    let result = function_taking_vec_of_structs(new_vec);
    result
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_TestStruct_vec")]
pub unsafe extern "C" fn drop_TestStruct_vec(_self: *mut Vec<TestStruct>) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____with_capacity_TestStruct_vec")]
pub unsafe extern "C" fn with_capacity_TestStruct_vec(capacity: usize) -> *mut Vec<TestStruct> {
    unsafe {
        let vec = Box::new(Vec::with_capacity(capacity));
        Box::into_raw(vec)
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____push_TestStruct_vec")]
pub unsafe extern "C" fn push_TestStruct_vec(_self: *mut Vec<TestStruct>, value: *mut TestStruct) {
    let value = unsafe { (*value).clone() };
    (&mut *_self).push(value);
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____len_TestStruct_vec")]
pub unsafe extern "C" fn len_TestStruct_vec(_self: *mut Vec<TestStruct>) -> usize {
    (&*_self).len()
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____get_TestStruct_vec")]
pub unsafe extern "C" fn get_TestStruct_vec(
    _self: *mut Vec<TestStruct>,
    index: usize,
) -> *mut TestStruct {
    {
        let v = (&*_self)[index].clone();
        Box::into_raw(Box::new(v))
    }
}
fn function_returning_vec_of_int() -> Vec<i32> {
    <[_]>::into_vec(::alloc::boxed::box_new([3, 2, 7, 8]))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_returning_vec_of_int")]
pub unsafe extern "C" fn ffi_wrapper_function_returning_vec_of_int() -> *mut Vec<i32> {
    let result = function_returning_vec_of_int();
    Box::into_raw(Box::new(result))
}
fn function_returning_vec_of_bool() -> Vec<bool> {
    <[_]>::into_vec(::alloc::boxed::box_new([true, false, true, true]))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_returning_vec_of_bool")]
pub unsafe extern "C" fn ffi_wrapper_function_returning_vec_of_bool() -> *mut Vec<bool> {
    let result = function_returning_vec_of_bool();
    Box::into_raw(Box::new(result))
}
fn function_returning_vec_of_structs() -> Vec<TestStruct2> {
    <[_]>::into_vec(::alloc::boxed::box_new([
        TestStruct2 { i32_field: 8 },
        TestStruct2 { i32_field: 11 },
    ]))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_returning_vec_of_structs")]
pub unsafe extern "C" fn ffi_wrapper_function_returning_vec_of_structs() -> *mut Vec<TestStruct2> {
    let result = function_returning_vec_of_structs();
    Box::into_raw(Box::new(result))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_TestStruct2_vec")]
pub unsafe extern "C" fn drop_TestStruct2_vec(_self: *mut Vec<TestStruct2>) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____with_capacity_TestStruct2_vec")]
pub unsafe extern "C" fn with_capacity_TestStruct2_vec(capacity: usize) -> *mut Vec<TestStruct2> {
    unsafe {
        let vec = Box::new(Vec::with_capacity(capacity));
        Box::into_raw(vec)
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____push_TestStruct2_vec")]
pub unsafe extern "C" fn push_TestStruct2_vec(
    _self: *mut Vec<TestStruct2>,
    value: *mut TestStruct2,
) {
    let value = unsafe { (*value).clone() };
    (&mut *_self).push(value);
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____len_TestStruct2_vec")]
pub unsafe extern "C" fn len_TestStruct2_vec(_self: *mut Vec<TestStruct2>) -> usize {
    (&*_self).len()
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____get_TestStruct2_vec")]
pub unsafe extern "C" fn get_TestStruct2_vec(
    _self: *mut Vec<TestStruct2>,
    index: usize,
) -> *mut TestStruct2 {
    {
        let v = (&*_self)[index].clone();
        Box::into_raw(Box::new(v))
    }
}
fn function_returning_vec_of_string() -> Vec<String> {
    <[_]>::into_vec(::alloc::boxed::box_new([
        "Hello".to_string(),
        "World".to_string(),
        "Rust".to_string(),
    ]))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_returning_vec_of_string")]
pub unsafe extern "C" fn ffi_wrapper_function_returning_vec_of_string() -> *mut Vec<String> {
    let result = function_returning_vec_of_string();
    Box::into_raw(Box::new(result))
}
impl Drop for TestStruct {
    fn drop(&mut self) {}
}
impl Drop for TestStruct2 {
    fn drop(&mut self) {}
}
#[repr(C)]
pub enum TestStatus {
    Active = 1,
    Inactive = 2,
    Pending = 3,
}
#[automatically_derived]
impl ::core::clone::Clone for TestStatus {
    #[inline]
    fn clone(&self) -> TestStatus {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for TestStatus {}
#[automatically_derived]
impl ::core::fmt::Debug for TestStatus {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                TestStatus::Active => "Active",
                TestStatus::Inactive => "Inactive",
                TestStatus::Pending => "Pending",
            },
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for TestStatus {}
#[automatically_derived]
impl ::core::cmp::PartialEq for TestStatus {
    #[inline]
    fn eq(&self, other: &TestStatus) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
pub fn get_status() -> TestStatus {
    TestStatus::Active
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___get_status")]
pub unsafe extern "C" fn ffi_wrapper_get_status() -> TestStatus {
    let result = get_status();
    result
}
pub fn get_inactive_status() -> TestStatus {
    TestStatus::Inactive
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___get_inactive_status")]
pub unsafe extern "C" fn ffi_wrapper_get_inactive_status() -> TestStatus {
    let result = get_inactive_status();
    result
}
pub fn assert_active(status: TestStatus) {
    match (&status, &TestStatus::Active) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___assert_active")]
pub unsafe extern "C" fn ffi_wrapper_assert_active(status: TestStatus) {
    let result = assert_active(status);
    result
}
pub fn assert_inactive(status: TestStatus) {
    match (&status, &TestStatus::Inactive) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___assert_inactive")]
pub unsafe extern "C" fn ffi_wrapper_assert_inactive(status: TestStatus) {
    let result = assert_inactive(status);
    result
}
pub fn function_taking_vec_of_enums(vec: Vec<TestStatus>) {
    match (
        &<[_]>::into_vec(::alloc::boxed::box_new([
            TestStatus::Active,
            TestStatus::Inactive,
            TestStatus::Pending,
            TestStatus::Active,
        ])),
        &vec,
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_taking_vec_of_enums")]
pub unsafe extern "C" fn ffi_wrapper_function_taking_vec_of_enums(vec: *mut Vec<TestStatus>) {
    let mut new_vec = Vec::new();
    std::mem::swap(&mut new_vec, unsafe { &mut (*vec) });
    let result = function_taking_vec_of_enums(new_vec);
    result
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_TestStatus_vec")]
pub unsafe extern "C" fn drop_TestStatus_vec(_self: *mut Vec<TestStatus>) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____with_capacity_TestStatus_vec")]
pub unsafe extern "C" fn with_capacity_TestStatus_vec(capacity: usize) -> *mut Vec<TestStatus> {
    unsafe {
        let vec = Box::new(Vec::with_capacity(capacity));
        Box::into_raw(vec)
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____push_TestStatus_vec")]
pub unsafe extern "C" fn push_TestStatus_vec(_self: *mut Vec<TestStatus>, value: TestStatus) {
    (&mut *_self).push(value);
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____len_TestStatus_vec")]
pub unsafe extern "C" fn len_TestStatus_vec(_self: *mut Vec<TestStatus>) -> usize {
    (&*_self).len()
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____get_TestStatus_vec")]
pub unsafe extern "C" fn get_TestStatus_vec(
    _self: *mut Vec<TestStatus>,
    index: usize,
) -> TestStatus {
    (&*_self)[index]
}
pub fn function_returning_vec_of_enums() -> Vec<TestStatus> {
    <[_]>::into_vec(::alloc::boxed::box_new([
        TestStatus::Pending,
        TestStatus::Active,
        TestStatus::Inactive,
        TestStatus::Active,
    ]))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_returning_vec_of_enums")]
pub unsafe extern "C" fn ffi_wrapper_function_returning_vec_of_enums() -> *mut Vec<TestStatus> {
    let result = function_returning_vec_of_enums();
    Box::into_raw(Box::new(result))
}
pub struct StructWithVecs {
    pub vec_of_ints: Vec<i32>,
    pub vec_of_bools: Vec<bool>,
    pub vec_of_strings: Vec<String>,
    pub vec_of_structs: Vec<TestStruct2>,
}
#[automatically_derived]
impl ::core::fmt::Debug for StructWithVecs {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field4_finish(
            f,
            "StructWithVecs",
            "vec_of_ints",
            &self.vec_of_ints,
            "vec_of_bools",
            &self.vec_of_bools,
            "vec_of_strings",
            &self.vec_of_strings,
            "vec_of_structs",
            &&self.vec_of_structs,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for StructWithVecs {
    #[inline]
    fn clone(&self) -> StructWithVecs {
        StructWithVecs {
            vec_of_ints: ::core::clone::Clone::clone(&self.vec_of_ints),
            vec_of_bools: ::core::clone::Clone::clone(&self.vec_of_bools),
            vec_of_strings: ::core::clone::Clone::clone(&self.vec_of_strings),
            vec_of_structs: ::core::clone::Clone::clone(&self.vec_of_structs),
        }
    }
}
#[automatically_derived]
impl ::core::default::Default for StructWithVecs {
    #[inline]
    fn default() -> StructWithVecs {
        StructWithVecs {
            vec_of_ints: ::core::default::Default::default(),
            vec_of_bools: ::core::default::Default::default(),
            vec_of_strings: ::core::default::Default::default(),
            vec_of_structs: ::core::default::Default::default(),
        }
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____StructWithVecs__get_vec_of_ints")]
pub unsafe extern "C" fn StructWithVecs_get_vec_of_ints(
    _self: *mut StructWithVecs,
) -> *mut Vec<i32> {
    unsafe { &mut (&mut *_self).vec_of_ints as *mut Vec<i32> }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____StructWithVecs__set_vec_of_ints")]
pub unsafe extern "C" fn StructWithVecs_set_vec_of_ints(
    _self: *mut StructWithVecs,
    value: *mut Vec<i32>,
) {
    unsafe {
        std::mem::swap(&mut (*value), &mut (&mut *_self).vec_of_ints);
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____StructWithVecs__get_vec_of_bools")]
pub unsafe extern "C" fn StructWithVecs_get_vec_of_bools(
    _self: *mut StructWithVecs,
) -> *mut Vec<bool> {
    unsafe { &mut (&mut *_self).vec_of_bools as *mut Vec<bool> }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____StructWithVecs__set_vec_of_bools")]
pub unsafe extern "C" fn StructWithVecs_set_vec_of_bools(
    _self: *mut StructWithVecs,
    value: *mut Vec<bool>,
) {
    unsafe {
        std::mem::swap(&mut (*value), &mut (&mut *_self).vec_of_bools);
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____StructWithVecs__get_vec_of_strings")]
pub unsafe extern "C" fn StructWithVecs_get_vec_of_strings(
    _self: *mut StructWithVecs,
) -> *mut Vec<String> {
    unsafe { &mut (&mut *_self).vec_of_strings as *mut Vec<String> }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____StructWithVecs__set_vec_of_strings")]
pub unsafe extern "C" fn StructWithVecs_set_vec_of_strings(
    _self: *mut StructWithVecs,
    value: *mut Vec<String>,
) {
    unsafe {
        std::mem::swap(&mut (*value), &mut (&mut *_self).vec_of_strings);
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____StructWithVecs__get_vec_of_structs")]
pub unsafe extern "C" fn StructWithVecs_get_vec_of_structs(
    _self: *mut StructWithVecs,
) -> *mut Vec<TestStruct2> {
    unsafe { &mut (&mut *_self).vec_of_structs as *mut Vec<TestStruct2> }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____StructWithVecs__set_vec_of_structs")]
pub unsafe extern "C" fn StructWithVecs_set_vec_of_structs(
    _self: *mut StructWithVecs,
    value: *mut Vec<TestStruct2>,
) {
    unsafe {
        std::mem::swap(&mut (*value), &mut (&mut *_self).vec_of_structs);
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____StructWithVecs__default")]
pub unsafe extern "C" fn StructWithVecs__default() -> *mut StructWithVecs {
    unsafe {
        let instance = Box::new(StructWithVecs::default());
        let ptr = Box::into_raw(instance);
        ptr
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____StructWithVecs__drop")]
pub unsafe extern "C" fn StructWithVecs_drop(_self: *mut StructWithVecs) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(export_name = "hiFfi____StructWithVecs__clone")]
pub unsafe extern "C" fn StructWithVecs_clone(_self: *mut StructWithVecs) -> *mut StructWithVecs {
    unsafe {
        let cloned: Box<StructWithVecs> = Box::new((*_self).clone());
        Box::into_raw(cloned)
    }
}
pub struct StructError(EnumError);
#[automatically_derived]
impl ::core::fmt::Debug for StructError {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_tuple_field1_finish(f, "StructError", &&self.0)
    }
}
impl std::fmt::Display for StructError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("StructError: {0}", self.0))
    }
}
impl std::error::Error for StructError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}
pub struct SimpleError;
#[automatically_derived]
impl ::core::fmt::Debug for SimpleError {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "SimpleError")
    }
}
impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("SimpleError"))
    }
}
impl std::error::Error for SimpleError {}
pub enum EnumError {
    VariantOne,
    VariantTwo(SimpleError),
}
#[automatically_derived]
impl ::core::fmt::Debug for EnumError {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            EnumError::VariantOne => ::core::fmt::Formatter::write_str(f, "VariantOne"),
            EnumError::VariantTwo(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "VariantTwo", &__self_0)
            }
        }
    }
}
impl std::fmt::Display for EnumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnumError::VariantOne => f.write_fmt(format_args!("EnumError: VariantOne")),
            EnumError::VariantTwo(_) => f.write_fmt(format_args!("EnumError: VariantTwo")),
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
pub fn function_with_primitive_result(error: bool) -> Result<i32, StructError> {
    if error {
        Err(StructError(EnumError::VariantTwo(SimpleError)))
    } else {
        Ok(123)
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_with_primitive_result")]
pub unsafe extern "C" fn ffi_wrapper_function_with_primitive_result(
    error: bool,
) -> *mut std::result::Result<i32, std::sync::Arc<dyn std::error::Error>> {
    let result = function_with_primitive_result(error);
    Box::into_raw(Box::new(result.map_err(|e| {
        std::sync::Arc::new(e) as std::sync::Arc<dyn std::error::Error>
    })))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_i32_result")]
pub unsafe extern "C" fn drop_i32_result(
    _self: *mut std::result::Result<i32, std::sync::Arc<dyn std::error::Error>>,
) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_i32_result")]
pub unsafe extern "C" fn unwrap_i32_result(
    _self: *mut std::result::Result<i32, std::sync::Arc<dyn std::error::Error>>,
) -> i32 {
    unsafe {
        match &*_self {
            Ok(value) => value.clone(),
            Err(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap on an Err value"));
            }
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_err_i32_result")]
pub unsafe extern "C" fn unwrap_err_i32_result(
    _self: *mut std::result::Result<i32, std::sync::Arc<dyn std::error::Error>>,
) -> *mut std::sync::Arc<dyn std::error::Error> {
    unsafe {
        match &*_self {
            Ok(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap_err on an Ok value"));
            }
            Err(err) => Box::into_raw(Box::new((*err).clone())),
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____is_err_i32_result")]
pub unsafe extern "C" fn is_err_i32_result(
    _self: *mut std::result::Result<i32, std::sync::Arc<dyn std::error::Error>>,
) -> bool {
    unsafe {
        match &*_self {
            Ok(_) => false,
            Err(_) => true,
        }
    }
}
pub fn function_with_bool_result(error: bool) -> Result<bool, SimpleError> {
    if error {
        Err(SimpleError)
    } else {
        Ok(true)
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_with_bool_result")]
pub unsafe extern "C" fn ffi_wrapper_function_with_bool_result(
    error: bool,
) -> *mut std::result::Result<bool, std::sync::Arc<dyn std::error::Error>> {
    let result = function_with_bool_result(error);
    Box::into_raw(Box::new(result.map_err(|e| {
        std::sync::Arc::new(e) as std::sync::Arc<dyn std::error::Error>
    })))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_bool_result")]
pub unsafe extern "C" fn drop_bool_result(
    _self: *mut std::result::Result<bool, std::sync::Arc<dyn std::error::Error>>,
) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_bool_result")]
pub unsafe extern "C" fn unwrap_bool_result(
    _self: *mut std::result::Result<bool, std::sync::Arc<dyn std::error::Error>>,
) -> bool {
    unsafe {
        match &*_self {
            Ok(value) => value.clone(),
            Err(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap on an Err value"));
            }
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_err_bool_result")]
pub unsafe extern "C" fn unwrap_err_bool_result(
    _self: *mut std::result::Result<bool, std::sync::Arc<dyn std::error::Error>>,
) -> *mut std::sync::Arc<dyn std::error::Error> {
    unsafe {
        match &*_self {
            Ok(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap_err on an Ok value"));
            }
            Err(err) => Box::into_raw(Box::new((*err).clone())),
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____is_err_bool_result")]
pub unsafe extern "C" fn is_err_bool_result(
    _self: *mut std::result::Result<bool, std::sync::Arc<dyn std::error::Error>>,
) -> bool {
    unsafe {
        match &*_self {
            Ok(_) => false,
            Err(_) => true,
        }
    }
}
pub fn function_with_string_result(error: bool) -> Result<String, EnumError> {
    if error {
        Err(EnumError::VariantOne)
    } else {
        Ok("No error".to_string())
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_with_string_result")]
pub unsafe extern "C" fn ffi_wrapper_function_with_string_result(
    error: bool,
) -> *mut std::result::Result<String, std::sync::Arc<dyn std::error::Error>> {
    let result = function_with_string_result(error);
    Box::into_raw(Box::new(result.map_err(|e| {
        std::sync::Arc::new(e) as std::sync::Arc<dyn std::error::Error>
    })))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_String_result")]
pub unsafe extern "C" fn drop_String_result(
    _self: *mut std::result::Result<String, std::sync::Arc<dyn std::error::Error>>,
) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_String_result")]
pub unsafe extern "C" fn unwrap_String_result(
    _self: *mut std::result::Result<String, std::sync::Arc<dyn std::error::Error>>,
) -> *mut std::string::String {
    unsafe {
        match &*_self {
            Ok(value) => Box::into_raw(Box::new(value.clone())),
            Err(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap on an Err value"));
            }
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_err_String_result")]
pub unsafe extern "C" fn unwrap_err_String_result(
    _self: *mut std::result::Result<String, std::sync::Arc<dyn std::error::Error>>,
) -> *mut std::sync::Arc<dyn std::error::Error> {
    unsafe {
        match &*_self {
            Ok(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap_err on an Ok value"));
            }
            Err(err) => Box::into_raw(Box::new((*err).clone())),
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____is_err_String_result")]
pub unsafe extern "C" fn is_err_String_result(
    _self: *mut std::result::Result<String, std::sync::Arc<dyn std::error::Error>>,
) -> bool {
    unsafe {
        match &*_self {
            Ok(_) => false,
            Err(_) => true,
        }
    }
}
pub fn function_with_struct_result(error: bool) -> Result<TestStruct2, StructError> {
    if error {
        Err(StructError(EnumError::VariantTwo(SimpleError)))
    } else {
        Ok(TestStruct2 { i32_field: 256 })
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_with_struct_result")]
pub unsafe extern "C" fn ffi_wrapper_function_with_struct_result(
    error: bool,
) -> *mut std::result::Result<TestStruct2, std::sync::Arc<dyn std::error::Error>> {
    let result = function_with_struct_result(error);
    Box::into_raw(Box::new(result.map_err(|e| {
        std::sync::Arc::new(e) as std::sync::Arc<dyn std::error::Error>
    })))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_TestStruct2_result")]
pub unsafe extern "C" fn drop_TestStruct2_result(
    _self: *mut std::result::Result<TestStruct2, std::sync::Arc<dyn std::error::Error>>,
) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_TestStruct2_result")]
pub unsafe extern "C" fn unwrap_TestStruct2_result(
    _self: *mut std::result::Result<TestStruct2, std::sync::Arc<dyn std::error::Error>>,
) -> *mut TestStruct2 {
    unsafe {
        match &*_self {
            Ok(value) => Box::into_raw(Box::new(value.clone())),
            Err(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap on an Err value"));
            }
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_err_TestStruct2_result")]
pub unsafe extern "C" fn unwrap_err_TestStruct2_result(
    _self: *mut std::result::Result<TestStruct2, std::sync::Arc<dyn std::error::Error>>,
) -> *mut std::sync::Arc<dyn std::error::Error> {
    unsafe {
        match &*_self {
            Ok(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap_err on an Ok value"));
            }
            Err(err) => Box::into_raw(Box::new((*err).clone())),
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____is_err_TestStruct2_result")]
pub unsafe extern "C" fn is_err_TestStruct2_result(
    _self: *mut std::result::Result<TestStruct2, std::sync::Arc<dyn std::error::Error>>,
) -> bool {
    unsafe {
        match &*_self {
            Ok(_) => false,
            Err(_) => true,
        }
    }
}
pub fn function_with_enum_result(error: bool) -> Result<TestStatus, EnumError> {
    if error {
        Err(EnumError::VariantOne)
    } else {
        Ok(TestStatus::Pending)
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_with_enum_result")]
pub unsafe extern "C" fn ffi_wrapper_function_with_enum_result(
    error: bool,
) -> *mut std::result::Result<TestStatus, std::sync::Arc<dyn std::error::Error>> {
    let result = function_with_enum_result(error);
    Box::into_raw(Box::new(result.map_err(|e| {
        std::sync::Arc::new(e) as std::sync::Arc<dyn std::error::Error>
    })))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_TestStatus_result")]
pub unsafe extern "C" fn drop_TestStatus_result(
    _self: *mut std::result::Result<TestStatus, std::sync::Arc<dyn std::error::Error>>,
) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_TestStatus_result")]
pub unsafe extern "C" fn unwrap_TestStatus_result(
    _self: *mut std::result::Result<TestStatus, std::sync::Arc<dyn std::error::Error>>,
) -> TestStatus {
    unsafe {
        match &*_self {
            Ok(value) => value.clone(),
            Err(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap on an Err value"));
            }
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_err_TestStatus_result")]
pub unsafe extern "C" fn unwrap_err_TestStatus_result(
    _self: *mut std::result::Result<TestStatus, std::sync::Arc<dyn std::error::Error>>,
) -> *mut std::sync::Arc<dyn std::error::Error> {
    unsafe {
        match &*_self {
            Ok(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap_err on an Ok value"));
            }
            Err(err) => Box::into_raw(Box::new((*err).clone())),
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____is_err_TestStatus_result")]
pub unsafe extern "C" fn is_err_TestStatus_result(
    _self: *mut std::result::Result<TestStatus, std::sync::Arc<dyn std::error::Error>>,
) -> bool {
    unsafe {
        match &*_self {
            Ok(_) => false,
            Err(_) => true,
        }
    }
}
pub fn function_with_vec_int_result(error: bool) -> Result<Vec<i32>, StructError> {
    if error {
        Err(StructError(EnumError::VariantTwo(SimpleError)))
    } else {
        Ok(<[_]>::into_vec(::alloc::boxed::box_new([10, 20, 30])))
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi___function_with_vec_int_result")]
pub unsafe extern "C" fn ffi_wrapper_function_with_vec_int_result(
    error: bool,
) -> *mut std::result::Result<Vec<"i32">, std::sync::Arc<dyn std::error::Error>> {
    let result = function_with_vec_int_result(error);
    Box::into_raw(Box::new(result.map_err(|e| {
        std::sync::Arc::new(e) as std::sync::Arc<dyn std::error::Error>
    })))
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____drop_i32_vec_result")]
pub unsafe extern "C" fn drop_i32_vec_result(
    _self: *mut std::result::Result<Vec<i32>, std::sync::Arc<dyn std::error::Error>>,
) {
    unsafe {
        if !_self.is_null() {
            let _ = Box::from_raw(_self);
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_vec_of_i32_result")]
pub unsafe extern "C" fn unwrap_vec_of_i32_result(
    _self: *mut std::result::Result<Vec<i32>, std::sync::Arc<dyn std::error::Error>>,
) -> *mut Vec<i32> {
    unsafe {
        match &*_self {
            Ok(value) => Box::into_raw(Box::new(value.clone())),
            Err(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap on an Err value"));
            }
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____unwrap_err_vec_of_i32_result")]
pub unsafe extern "C" fn unwrap_err_vec_of_i32_result(
    _self: *mut std::result::Result<Vec<i32>, std::sync::Arc<dyn std::error::Error>>,
) -> *mut std::sync::Arc<dyn std::error::Error> {
    unsafe {
        match &*_self {
            Ok(_) => {
                ::core::panicking::panic_fmt(format_args!("Called unwrap_err on an Ok value"));
            }
            Err(err) => Box::into_raw(Box::new((*err).clone())),
        }
    }
}
#[doc(hidden)]
#[unsafe(no_mangle)]
#[unsafe(export_name = "hiFfi____is_err_vec_of_i32_result")]
pub unsafe extern "C" fn is_err_vec_of_i32_result(
    _self: *mut std::result::Result<Vec<i32>, std::sync::Arc<dyn std::error::Error>>,
) -> bool {
    unsafe {
        match &*_self {
            Ok(_) => false,
            Err(_) => true,
        }
    }
}
