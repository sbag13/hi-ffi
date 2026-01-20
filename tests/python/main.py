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
    TestStruct.static_method_taking_primitives(20, True)
    TestStruct.static_method_taking_string("static method string")
    static_arg = TestStruct2()
    static_arg.i32_field = 77
    TestStruct.static_method_taking_struct(static_arg)

    assert TestStruct.static_method_returning_primitive() == 22
    assert (
        TestStruct.static_method_returning_string()
        == "String returned from Rust static method"
    )
    assert TestStruct.static_method_returning_struct().i32_field == 77
    assert TestStruct.static_combo_method("first", "second", False) == "second"

    combo_arg1 = TestStruct2()
    combo_arg1.i32_field = 300
    combo_arg2 = TestStruct2()
    combo_arg2.i32_field = 400
    combo_result = TestStruct.static_combo_struct_method(combo_arg1, combo_arg2)
    assert combo_result.i32_field == 300


def methods_tests():
    s = TestStruct()

    s2 = TestStruct2()
    s2.i32_field = 55

    s.public_method()
    s.public_method_taking_primitives(10, False)
    s.public_method_taking_string("Test string from caller")
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
    s.i32_field = 42
    assert s.i32_field == 42

    assert s.f32_field == 0.0
    s.f32_field = 3.14
    assert abs(s.f32_field - 3.14) < 0.0001

    assert s.bool_field == False
    s.bool_field = True
    assert s.bool_field == True

    assert s.string_field == ""
    s.string_field = "Hello, World!"
    assert s.string_field == "Hello, World!"
    s.string_field = "Hello, Rust!"
    assert s.string_field == "Hello, Rust!"
    s.string_field = "Hello, C++!"
    assert s.string_field == "Hello, C++!"

    struct2 = s.struct_field
    struct2.i32_field = 43
    assert struct2.i32_field == 43

    struct2_other = TestStruct2()
    struct2_other.i32_field = 44
    s.struct_field = struct2_other
    assert s.struct_field.i32_field == 44

    # vector getters/setters
    struct_with_vecs = StructWithVecs()

    assert len(struct_with_vecs.vec_of_ints) == 0
    struct_with_vecs.vec_of_ints = [3, 5, 7]
    assert struct_with_vecs.vec_of_ints == [3, 5, 7]

    assert len(struct_with_vecs.vec_of_bools) == 0
    struct_with_vecs.vec_of_bools = [True, True, False]
    assert struct_with_vecs.vec_of_bools == [True, True, False]

    assert len(struct_with_vecs.vec_of_strings) == 0
    struct_with_vecs.vec_of_strings = ["new", "vec"]
    assert struct_with_vecs.vec_of_strings == ["new", "vec"]

    assert len(struct_with_vecs.vec_of_structs) == 0
    ts2 = TestStruct2()
    ts2.i32_field = 567
    struct_with_vecs.vec_of_structs = [TestStruct2(), ts2]
    assert len(struct_with_vecs.vec_of_structs) == 2
    assert struct_with_vecs.vec_of_structs[0].i32_field == 0
    assert struct_with_vecs.vec_of_structs[1].i32_field == 567


def functions_tests():
    s = TestStruct()

    take_status_before_it_is_defined(TestStatus.Pending)
    ts1 = TestStruct()
    ts1.i32_field = -5
    assert (
        take_struct_and_return_status_before_they_are_defined(ts1) == TestStatus.Pending
    )

    # check if doesn't crash
    simple_function()
    function_with_primitive_args(100, True)
    function_with_string_arg("Hello, World!")
    function_with_primitive_and_string_arg(42, False, "Complex function!")
    ts2 = TestStruct2()
    ts2.i32_field = 55
    function_taking_struct(ts2)

    # assert return values
    assert function_return_primitive() == 42
    assert function_return_float() == 5.21
    assert function_return_string() == "String returned from Rust"
    assert function_return_negated_bool(True) == False
    assert function_return_negated_bool(False) == True
    assert combo_function("str1", "str2", True, TestStruct()) == "str1"
    struct2_from_function = function_returning_struct()
    assert struct2_from_function.i32_field == 48

    s = TestStruct()
    s.i32_field = 49
    combo_struct_function_result = combo_struct_function(s, TestStruct(), TestStruct2())
    assert combo_struct_function_result.i32_field == 49


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


def result_tests():
    print("result_tests")
    try:
        _ = function_with_primitive_result(True)
    except Exception as e:
        assert str(e) == "StructError: EnumError: VariantTwo"
        source = e.source()
        assert str(source) == "EnumError: VariantTwo"
        source_of_source = source.source()
        assert str(source_of_source) == "SimpleError"
        assert source_of_source.source() is None

    assert function_with_primitive_result(False) == 123
    assert function_with_string_result(False) == "No error"
    assert function_with_struct_result(False).i32_field == 256
    assert function_with_enum_result(False) == TestStatus.Pending
    assert function_with_vec_int_result(False) == [10, 20, 30]
    assert function_with_vec_bool_result(False) == [True, False, False]
    assert function_with_vec_string_result(False) == ["One", "Two", "Three"]
    ok_structs = function_with_vec_struct_result(False)
    assert len(ok_structs) == 2
    assert ok_structs[0].i32_field == 512
    assert ok_structs[1].i32_field == 1024
    ok_enums = function_with_vec_enum_result(False)
    assert len(ok_enums) == 3
    assert ok_enums[0] == TestStatus.Pending
    assert ok_enums[1] == TestStatus.Active
    assert ok_enums[2] == TestStatus.Inactive
    function_with_unit_expression_result()  # just check no exception

    ts1 = TestStruct()
    try:
        _ = ts1.method_with_int_result(True)
    except Exception as e:
        assert str(e) == "StructError: EnumError: VariantTwo"

    assert ts1.method_with_int_result(False) == 16
    assert TestStruct.static_method_with_bool_result(False)
    assert ts1.method_with_string_result() == "Ok!"
    assert TestStruct.static_method_with_struct_result().i32_field == 267
    assert ts1.method_with_enum_result() == TestStatus.Pending
    assert ts1.method_with_vec_of_ints_result() == [1, 2, 7]
    assert TestStruct.static_method_with_vec_of_bools_result() == [True, False, False]
    assert ts1.method_with_vec_of_strings_result() == ["some", "string"]
    ok_vec_of_structs = TestStruct.static_method_with_vec_of_structs_result()
    assert len(ok_vec_of_structs) == 2
    assert ok_vec_of_structs[0].i32_field == 2
    assert ok_vec_of_structs[1].i32_field == -5
    assert ts1.method_with_vec_of_bools_result() == [False, True, True, True]
    assert TestStruct.static_method_with_vec_of_enum_result() == [
        TestStatus.Inactive,
        TestStatus.Pending,
        TestStatus.Active,
    ]


if __name__ == "__main__":
    struct_tests()
    functions_tests()
    methods_tests()
    static_methods_tests()
    vector_tests()
    vector_methods_tests()
    enum_tests()
    result_tests()
    print("All tests passed!")
