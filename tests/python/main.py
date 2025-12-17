import ctypes
import os
import sys

sys.path.insert(
    0,
    os.path.abspath(os.path.join(os.path.dirname(__file__), "../generated_code")),
)

from python_ffi import ffi_init
from python_ffi import *
from python_ffi.TestStruct import TestStruct
from python_ffi.TestStruct2 import TestStruct2


ext = "so"  # Change to 'dylib' for MacOS, 'dll' for Windows
lib_path = os.path.join(
    os.path.dirname(__file__), "..", "target", "debug", f"libtests.{ext}"
)
lib = ctypes.CDLL(lib_path)

ffi_init(lib)


def static_methods_tests():
    TestStruct.static_method()
    TestStruct.static_method_taking_primitives(48, True)
    TestStruct.static_method_taking_string("Hello from Python!")
    TestStruct.static_method_taking_struct(TestStruct2())

    assert TestStruct.static_method_returning_primitive() == 22
    assert (
        TestStruct.static_method_returning_string()
        == "String returned from Rust static method"
    )
    assert TestStruct.static_method_returning_struct().i32_field == 77
    assert TestStruct.static_combo_method("s1", "s2", False) == "s2"

    s1 = TestStruct2()
    s1.i32_field = 1
    assert TestStruct.static_combo_struct_method(s1, TestStruct2()).i32_field == 1


def methods_tests():
    s = TestStruct()

    s2 = TestStruct2()
    s2.i32_field = 6

    s.public_method()
    s.public_method_taking_primitives(48, True)
    s.public_method_taking_string("Hello from Python!")
    s.public_method_taking_struct(s2)

    assert s.public_method_returning_primitive() == 24
    assert s.public_method_returning_bool() == True
    assert s.public_method_returning_string() == "String returned from Rust method"
    assert s.public_method_returning_struct().i32_field == 99
    assert s.combo_method("str1", "str2", False) == "str2"


def struct_tests():
    s = TestStruct()

    # check props
    assert s.i32_field == 0
    s.i32_field = 123
    assert s.i32_field == 123

    assert s.bool_field == False
    s.bool_field = True
    assert s.bool_field == True

    assert s.f32_field == 0.0
    s.f32_field = 3.14
    assert abs(s.f32_field - 3.14) < 0.0001

    assert s.string_field == ""
    s.string_field = "Hello, Python!"
    assert s.string_field == "Hello, Python!"

    assert s.struct_field.i32_field == 0
    new_struct_field = TestStruct2()
    new_struct_field.i32_field = 5
    s.struct_field = new_struct_field
    assert s.struct_field.i32_field == 5


def functions_tests():
    s = TestStruct()

    # check if doesn't crash
    simple_function()
    function_with_primitive_args(48, True)
    function_with_string_arg("Hello from Python!")
    function_with_primitive_and_string_arg(99, False, "Another string")
    function_taking_struct(TestStruct2())

    # assert return values
    assert function_return_primitive() == 42
    assert function_return_float() == 5.21
    assert function_return_string() == "String returned from Rust"
    assert function_return_negated_bool(True) == False
    assert function_return_negated_bool(False) == True
    assert "second" == combo_function("first", "second", False, TestStruct())
    assert function_returning_struct().i32_field == 48

    s = TestStruct()
    s.i32_field = 43
    assert combo_struct_function(s, TestStruct(), TestStruct2()).i32_field == 43


def vector_tests():
    # Test vector arguments
    function_taking_vec_of_primitives([1, 2, 3, 4, 5])
    function_taking_vec_of_bools([True, False, True, True])
    function_taking_vec_of_strings(["Hello, Rust!", "Hello, C++!", "Hello, Python!", "Hello, Swift!"])
    
    # Test vector of structs
    s1 = TestStruct()
    s1.i32_field = 15
    s2 = TestStruct()
    s2.i32_field = 17
    function_taking_vec_of_structs([s1, s2])

    # Test vector return values
    assert function_returning_vec_of_int() == [3, 2, 7, 8]
    assert function_returning_vec_of_bool() == [True, False, True, True]
    assert function_returning_vec_of_string() == ["Hello", "World", "Rust"]
    
    returned_structs = function_returning_vec_of_structs()
    assert len(returned_structs) == 2
    assert returned_structs[0].i32_field == 8
    assert returned_structs[1].i32_field == 11


if __name__ == "__main__":
    struct_tests()
    functions_tests()
    methods_tests()
    static_methods_tests()
    vector_tests()
    print("All tests passed!")
