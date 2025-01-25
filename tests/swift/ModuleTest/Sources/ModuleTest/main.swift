

import Foundation
import FfiModule

// TODO assertions

func run () {
    print("Swift FFI Test Suite")

    // Functions

    simple_function()
    print("simple_function called")

    print("Calling function_with_primitive_args")
    function_with_primitive_args(1, true)

    print("Calling function_with_string_arg")
    function_with_string_arg("Hello, World!")

    print("Calling function_with_primitive_and_string_arg")
    function_with_primitive_and_string_arg(1, false, "Hello, World!")

    print("Calling function_return_primitive")
    let primitive_result = function_return_primitive()
    print("Primitive result: \(primitive_result)")

    print("Calling function_return_string")
    let str_result = function_return_string()
    print(str_result)

    print(combo_function("Combo!", "Don't print me", true))

    // Structs basics

    print("Creating a struct")
    let s = TestStruct()

    print("Getting i32_field")
    let i32_field = s.i32_field
    print("i32_field: \(i32_field)")

    print("Setting i32_field")
    s.i32_field = 42
    print("i32_field: \(s.i32_field)")

    print("Getting string field")
    let string_field = s.string_field
    print("string_field (should be empty): \(string_field)")

    print("Setting string field")
    s.string_field = "Hello, World!"
    print("updated string_field: \(s.string_field)")

    // Structs methods from impl block

    print("Calling simple method")
    s.public_method()

    print("Calling method with primitives")
    s.public_method_taking_primitives(1, true)

    print("Calling method with string")
    s.public_method_taking_string("Hello, World!")

    print("Calling public method returning primitive")
    let public_method_returning_primitive = s.public_method_returning_primitive()
    print("public_method_returning_primitive: \(public_method_returning_primitive)")

    print("Calling method returning string")
    let public_method_returning_string = s.public_method_returning_string()
    print("public_method_returning_string: \(public_method_returning_string)")

    print("Calling combo method")
    let combo_method_result = s.combo_method("Combo!", "Don't print me", true)
    print("combo_method_result: \(combo_method_result)")

    // Structs static methods from impl blocks

    print("Calling simple static method")
    TestStruct.static_method()

    print("Calling static method with primitives")
    TestStruct.static_method_taking_primitives(1, true)

    print("Calling static method with string")
    TestStruct.static_method_taking_string("Hello, World!")

    print("Calling static method returning primitive")
    let static_method_returning_primitive = TestStruct.static_method_returning_primitive()
    print("static_method_returning_primitive: \(static_method_returning_primitive)")

    print("Calling static method returning string")
    let static_method_returning_string = TestStruct.static_method_returning_string()
    print("static_method_returning_string: \(static_method_returning_string)")

    print("Calling static combo method")
    let static_combo_method_result = TestStruct.static_combo_method("Combo!", "Don't print me", true)
    print("static_combo_method_result: \(static_combo_method_result)")
}

run()