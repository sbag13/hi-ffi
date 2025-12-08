import Foundation
import FfiModule

func assert_struct_basics() {
    let s = TestStruct()
    assert(s.i32_field == 0, "Default i32_field should be 0")
    s.i32_field = 42
    assert(s.i32_field == 42, "i32_field should be 42 after set")
    assert(s.string_field == "", "Default string_field should be empty")
    s.string_field = "Hello, World!"
    assert(s.string_field == "Hello, World!", "string_field should be updated")
    let new_struct_field = TestStruct2()
    new_struct_field.i32_field = 999
    s.struct_field = new_struct_field
    assert(s.struct_field.i32_field == 999, "struct_field.i32_field should be 999")
}

func assert_functions() {
    simple_function()
    function_with_primitive_args(1, true)
    function_with_string_arg("Hello, World!")
    function_with_primitive_and_string_arg(1, false, "Hello, World!")
    assert(function_return_primitive() == 42, "function_return_primitive should return 42")
    assert(function_return_string() == "String returned from Rust", "function_return_string should return correct string")
    let s = TestStruct()
    assert(combo_function("Combo!", "Don't print me", true, s) == "Combo!", "combo_function should return correct string")
    let s2 = function_returning_struct()
    assert(s2.i32_field == 48, "function_returning_struct should return struct with i32_field 48")
    function_taking_struct(s2)
    let combo_struct_result = combo_struct_function(s, TestStruct(), s2)
    assert(combo_struct_result.i32_field == s.i32_field, "combo_struct_result i32_field should match input")
}

func assert_struct_methods() {
    let s = TestStruct()
    s.public_method()
    s.public_method_taking_primitives(1, true)
    s.public_method_taking_string("Hello, World!")
    let struct_from_method = s.public_method_returning_struct()
    s.public_method_taking_struct(struct_from_method)
    assert(s.public_method_returning_primitive() == 24, "public_method_returning_primitive should return 24")
    assert(s.public_method_returning_string() == "String returned from Rust method", "public_method_returning_string should return correct string")
    assert(s.combo_method("Combo!", "Don't print me", true) == "Combo!", "combo_method should return correct string")
}

func assert_struct_static_methods() {
    let s2 = function_returning_struct()
    TestStruct.static_method()
    TestStruct.static_method_taking_primitives(1, true)
    TestStruct.static_method_taking_string("Hello, World!")
    assert(TestStruct.static_method_returning_primitive() == 22, "static_method_returning_primitive should return 22")
    assert(TestStruct.static_method_returning_string() == "String returned from Rust static method", "static_method_returning_string should return correct string")
    let static_struct = TestStruct.static_method_returning_struct()
    assert(static_struct.i32_field == 77, "static_method_returning_struct should return struct with i32_field 77")
    let static_combo_struct_result = TestStruct.static_combo_struct_method(s2, static_struct)
    assert(static_combo_struct_result.i32_field == s2.i32_field, "static_combo_struct_result i32_field should match input")
    assert(TestStruct.static_combo_method("Combo!", "Don't print me", true) == "Combo!", "static_combo_method should return correct string")
}

func run() {
    assert_struct_basics()
    assert_functions()
    assert_struct_methods()
    assert_struct_static_methods()
    print("All assertions passed.")
}

run()
