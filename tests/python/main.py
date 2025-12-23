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
from python_ffi.TestStatus import TestStatus
from python_ffi.vec_i32 import i32Vec
from python_ffi.vec_String import StringVec
from python_ffi.vec_TestStruct2 import TestStruct2Vec
from python_ffi.vec_TestStatus import TestStatusVec
from python_ffi.StructWithVecs import StructWithVecs


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

    # vector getters/setters
    struct_with_vecs = StructWithVecs()

    assert len(struct_with_vecs.vec_of_ints) == 0
    struct_with_vecs.vec_of_ints = [35, 55]
    assert struct_with_vecs.vec_of_ints == [35, 55]

    assert len(struct_with_vecs.vec_of_bools) == 0
    struct_with_vecs.vec_of_bools = [True, True, False]
    assert struct_with_vecs.vec_of_bools == [True, True, False]

    assert len(struct_with_vecs.vec_of_strings) == 0
    struct_with_vecs.vec_of_strings = ["Hello", "from", "Python"]
    assert struct_with_vecs.vec_of_strings == ["Hello", "from", "Python"]

    assert len(struct_with_vecs.vec_of_structs) == 0
    ts2 = TestStruct2()
    ts2.i32_field = 5
    struct_with_vecs.vec_of_structs = [TestStruct2(), ts2]
    assert len(struct_with_vecs.vec_of_structs) == 2
    assert struct_with_vecs.vec_of_structs[0].i32_field == 0
    assert struct_with_vecs.vec_of_structs[1].i32_field == 5


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
    function_taking_vec_of_strings(
        ["Hello, Rust!", "Hello, C++!", "Hello, Python!", "Hello, Swift!"]
    )

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

    # Test vector of enums
    function_taking_vec_of_enums(
        [TestStatus.Active, TestStatus.Inactive, TestStatus.Pending, TestStatus.Active]
    )

    returned_enums = function_returning_vec_of_enums()
    assert len(returned_enums) == 4
    assert returned_enums == [
        TestStatus.Pending,
        TestStatus.Active,
        TestStatus.Inactive,
        TestStatus.Active,
    ]


def vector_methods_tests():
    print("vector_methods_tests")

    test_struct = TestStruct()

    # Test instance methods with vector arguments (now using native lists)
    vec_primitives = [1, 2, 3, 4, 5]
    test_struct.public_method_taking_vec_of_primitives(vec_primitives)

    vec_strings = ["Hello", "World"]
    test_struct.public_method_taking_vec_of_strings(vec_strings)

    s1 = TestStruct2()
    s1.i32_field = 42
    s2 = TestStruct2()
    s2.i32_field = 24
    vec_structs = [s1, s2]
    test_struct.public_method_taking_vec_of_structs(vec_structs)

    # Test static methods with vector arguments (now using native lists)
    static_vec_primitives = [6, 7, 8, 9, 10]
    TestStruct.static_method_taking_vec_of_primitives(static_vec_primitives)

    static_vec_strings = ["Static", "Method"]
    TestStruct.static_method_taking_vec_of_strings(static_vec_strings)

    s3 = TestStruct2()
    s3.i32_field = 100
    s4 = TestStruct2()
    s4.i32_field = 200
    static_vec_structs = [s3, s4]
    TestStruct.static_method_taking_vec_of_structs(static_vec_structs)

    # Test instance methods returning vectors
    returned_primitives = test_struct.public_method_returning_vec_of_primitives()
    expected_primitives = [10, 20, 30, 40, 50]
    assert (
        returned_primitives == expected_primitives
    ), f"Expected {expected_primitives}, got {returned_primitives}"

    returned_strings = test_struct.public_method_returning_vec_of_strings()
    expected_strings = ["Method", "Vector", "Return"]
    assert (
        returned_strings == expected_strings
    ), f"Expected {expected_strings}, got {returned_strings}"

    returned_structs = test_struct.public_method_returning_vec_of_structs()
    assert (
        len(returned_structs) == 2
    ), f"Expected 2 structs, got {len(returned_structs)}"
    assert (
        returned_structs[0].i32_field == 300
    ), f"Expected 300, got {returned_structs[0].i32_field}"
    assert (
        returned_structs[1].i32_field == 400
    ), f"Expected 400, got {returned_structs[1].i32_field}"

    # Test static methods returning vectors
    static_returned_primitives = TestStruct.static_method_returning_vec_of_primitives()
    static_expected_primitives = [60, 70, 80, 90, 100]
    assert (
        static_returned_primitives == static_expected_primitives
    ), f"Expected {static_expected_primitives}, got {static_returned_primitives}"

    static_returned_strings = TestStruct.static_method_returning_vec_of_strings()
    static_expected_strings = ["Static", "Method", "Vector"]
    assert (
        static_returned_strings == static_expected_strings
    ), f"Expected {static_expected_strings}, got {static_returned_strings}"

    static_returned_structs = TestStruct.static_method_returning_vec_of_structs()
    assert (
        len(static_returned_structs) == 2
    ), f"Expected 2 structs, got {len(static_returned_structs)}"
    assert (
        static_returned_structs[0].i32_field == 500
    ), f"Expected 500, got {static_returned_structs[0].i32_field}"
    assert (
        static_returned_structs[1].i32_field == 600
    ), f"Expected 600, got {static_returned_structs[1].i32_field}"


def enum_tests():
    # Test enum values
    print(f"TestStatus.Active = {TestStatus.Active}")
    print(f"TestStatus.Inactive = {TestStatus.Inactive}")
    print(f"TestStatus.Pending = {TestStatus.Pending}")

    # Test function returning enum
    status = get_status()
    assert status == TestStatus.Active, f"Expected Active, got {status}"

    # Test function returning different enum
    inactive_status = get_inactive_status()
    assert (
        inactive_status == TestStatus.Inactive
    ), f"Expected Inactive, got {inactive_status}"

    # Test function taking enum parameter
    assert_active(TestStatus.Active)
    assert_inactive(TestStatus.Inactive)


if __name__ == "__main__":
    struct_tests()
    functions_tests()
    methods_tests()
    static_methods_tests()
    vector_tests()
    vector_methods_tests()
    enum_tests()
    print("All tests passed!")
