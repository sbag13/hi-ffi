

import Foundation
import FfiModule

func run () {
    print("Swift FFI Test Suite")

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

    print("Creating a struct")
    let s = TestStruct()

    print("Getting i32_field")
    let i32_field = s.i32_field
    print("i32_field: \(i32_field)")

    print("Setting i32_field")
    s.i32_field = 42
    print("i32_field: \(s.i32_field)")

    print(combo_function("Combo!", "Don't print me", true))
}

run()