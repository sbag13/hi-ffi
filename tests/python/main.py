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
from python_ffi.option_i32 import i32Option
from python_ffi.option_bool import boolOption
from python_ffi.option_String import StringOption
from python_ffi.option_TestStatus import TestStatusOption
from python_ffi.option_TestStruct2 import TestStruct2Option
from python_ffi.StructWithOptions import StructWithOptions
from python_ffi.RustTrait import RustTrait
from typing import Protocol, runtime_checkable


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

    assert len(struct_with_vecs.vec_of_enums) == 0
    struct_with_vecs.vec_of_enums = [TestStatus.Active, TestStatus.Pending]
    assert struct_with_vecs.vec_of_enums == [TestStatus.Active, TestStatus.Pending]


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


def option_tests():
    print("option_tests")

    # Test function taking Option arguments with Some values
    function_taking_some_int(10, True)
    function_taking_some_int(None, False)
    function_taking_some_bool(True, True)
    function_taking_some_bool(None, False)
    function_taking_some_string("Some string", True)
    function_taking_some_string(None, False)
    function_taking_some_enum(TestStatus.Pending, True)
    function_taking_some_enum(None, False)
    struct_arg = TestStruct()
    struct_arg.i32_field = 567
    function_taking_some_struct(struct_arg, True)
    function_taking_some_struct(None, False)

    # Test function returning Option values (Some)
    opt_int = function_returning_opt_int(True)
    assert opt_int is not None
    assert opt_int == 100

    opt_bool = function_returning_opt_bool(True)
    assert opt_bool is not None
    assert opt_bool == True

    opt_string = function_returning_opt_string(True)
    assert opt_string is not None
    assert opt_string == "Some Rust String"

    opt_enum = function_returning_opt_enum(True)
    assert opt_enum is not None
    assert opt_enum == TestStatus.Pending

    opt_struct = function_returning_opt_struct(True)
    assert opt_struct is not None
    assert opt_struct.i32_field == 234

    # Test function returning None (None case)
    opt_int_none = function_returning_opt_int(False)
    assert opt_int_none is None

    opt_bool_none = function_returning_opt_bool(False)
    assert opt_bool_none is None

    opt_string_none = function_returning_opt_string(False)
    assert opt_string_none is None

    opt_enum_none = function_returning_opt_enum(False)
    assert opt_enum_none is None

    opt_struct_none = function_returning_opt_struct(False)
    assert opt_struct_none is None

    # Test instance methods with Option arguments
    test_struct = TestStruct()
    test_struct.method_taking_opt_int(20, True)
    test_struct.method_taking_opt_int(None, False)
    test_struct.method_taking_opt_bool(False, True)
    test_struct.method_taking_opt_bool(None, False)
    struct_arg2 = TestStruct2()
    struct_arg2.i32_field = 789
    test_struct.method_taking_opt_struct(struct_arg2, True)
    test_struct.method_taking_opt_struct(None, False)

    # Test instance methods returning Option
    result_int = test_struct.method_returning_opt_int(True)
    assert result_int is not None
    assert result_int == 30

    result_string = test_struct.method_returning_opt_string(True)
    assert result_string is not None
    assert result_string == "Optional string from Rust"

    result_struct = test_struct.method_returning_opt_struct(True)
    assert result_struct is not None
    assert result_struct.i32_field == 654

    # Test instance methods returning None
    result_int_none = test_struct.method_returning_opt_int(False)
    assert result_int_none is None

    result_string_none = test_struct.method_returning_opt_string(False)
    assert result_string_none is None

    result_struct_none = test_struct.method_returning_opt_struct(False)
    assert result_struct_none is None

    # Test static methods with Option
    TestStruct.static_method_taking_opt_string("Optional string", True)
    TestStruct.static_method_taking_opt_string(None, False)
    TestStruct.static_method_taking_opt_enum(TestStatus.Active, True)
    TestStruct.static_method_taking_opt_enum(None, False)

    static_result_bool = TestStruct.static_method_returning_opt_bool(True)
    assert static_result_bool is not None
    assert static_result_bool == False

    static_result_bool_none = TestStruct.static_method_returning_opt_bool(False)
    assert static_result_bool_none is None

    static_result_enum = TestStruct.static_method_returning_opt_enum(True)
    assert static_result_enum is not None
    assert static_result_enum == TestStatus.Inactive

    static_result_enum_none = TestStruct.static_method_returning_opt_enum(False)
    assert static_result_enum_none is None

    # Test struct with Option fields
    struct_with_options = StructWithOptions()

    assert struct_with_options.opt_int is None
    struct_with_options.opt_int = 555
    assert struct_with_options.opt_int == 555

    assert struct_with_options.opt_bool is None
    struct_with_options.opt_bool = True
    assert struct_with_options.opt_bool == True

    assert struct_with_options.opt_string is None
    struct_with_options.opt_string = "Struct with options"
    assert struct_with_options.opt_string == "Struct with options"

    assert struct_with_options.opt_enum is None
    struct_with_options.opt_enum = TestStatus.Active
    assert struct_with_options.opt_enum == TestStatus.Active

    assert struct_with_options.opt_struct is None
    some_ts2 = TestStruct2()
    some_ts2.i32_field = 321
    struct_with_options.opt_struct = some_ts2
    assert struct_with_options.opt_struct is not None
    assert struct_with_options.opt_struct.i32_field == 321


class PythonTraitImpl:
    def trait_simple_fn(self):
        print("Hello from python")

    def trait_fn_with_simple_args(self, i: int, f: float, e: TestStatus, b: bool):
        assert i == 42
        assert f == 4.2
        assert e == TestStatus.Pending
        assert b == True

    def trait_fn_with_string_arg(self, s: str):
        assert s == "Hello from trait object"

    def trait_fn_with_struct_arg(self, s: TestStruct):
        assert s.i32_field == 123

    def trait_fn_with_vec_of_primitives(self, vec: List[int]):
        assert vec == [1, 2, 3, 4, 5]

    def trait_fn_with_vec_of_bools(self, vec: List[bool]):
        assert vec == [True, False, True]

    def trait_fn_with_vec_of_enums(self, vec: List[TestStatus]):
        assert vec == [TestStatus.Active, TestStatus.Inactive]

    def trait_fn_with_vec_of_strings(self, vec: List[str]):
        assert vec == ["Hello", "Trait", "Object"]

    def trait_fn_with_vec_of_structs(self, vec: List[TestStruct2]):
        assert len(vec) == 2
        assert vec[0].i32_field == 321
        assert vec[1].i32_field == 654

    def trait_fn_with_options(
        self,
        opt_int: Optional[int],
        opt_string: Optional[str],
        opt_bool: Optional[bool],
        opt_enum: Optional[TestStatus],
        opt_struct: Optional[TestStruct2],
    ):
        assert opt_int == 42
        assert opt_string == "Hello from trait object"
        assert opt_bool == False
        assert opt_enum == TestStatus.Pending
        assert opt_struct.i32_field == 789

    def trait_fn_return_int(self) -> int:
        return 12345

    def trait_fn_return_bool(self) -> bool:
        return True

    def trait_fn_return_string(self) -> str:
        return "String from trait object"

    def trait_fn_return_struct(self) -> TestStruct2:
        s = TestStruct2()
        s.i32_field = 987
        return s

    def trait_fn_return_enum(self) -> TestStatus:
        return TestStatus.Inactive

    def trait_fn_return_vec_of_primitives(self) -> List[int]:
        return [10, 20, 30]

    def trait_fn_return_vec_of_bools(self) -> List[bool]:
        return [True, False, True, True]

    def trait_fn_return_vec_of_enums(self) -> List[TestStatus]:
        return [TestStatus.Active, TestStatus.Inactive, TestStatus.Pending]

    def trait_fn_return_vec_of_strings(self) -> List[str]:
        return ["Hello", "from", "trait", "object"]

    def trait_fn_return_vec_of_structs(self) -> List[TestStruct2]:
        s1 = TestStruct2()
        s1.i32_field = 111
        s2 = TestStruct2()
        s2.i32_field = 222
        return [s1, s2]

    def trait_fn_returning_option_int(self, some: bool) -> Optional[int]:
        if some:
            return 555
        return None

    def trait_fn_returning_option_bool(self, some: bool) -> Optional[bool]:
        if some:
            return False
        return None

    def trait_fn_returning_option_string(self, some: bool) -> Optional[str]:
        if some:
            return "Some string"
        return None

    def trait_fn_returning_option_enum(self, some: bool) -> Optional[TestStatus]:
        if some:
            return TestStatus.Pending
        return None

    def trait_fn_returning_option_struct(self, some: bool) -> Optional[TestStruct2]:
        if some:
            s = TestStruct2()
            s.i32_field = 789
            return s
        return None

    def __del__(self):
        print("Python delete called for PythonTraitImpl")


def trait_tests():
    print("Trait tests")

    impl = PythonTraitImpl()
    function_taking_trait_object(impl)

    # make sure that impl is still usable after passing to Rust
    assert impl.trait_fn_return_int() == 12345

    # Test returning trait objects from Rust
    rust_trait_obj = function_returning_trait_object()
    
    # Test calling simple method with no args
    rust_trait_obj.trait_simple_fn()
    
    # Test calling method with simple args (int, float, enum, bool)
    rust_trait_obj.trait_fn_with_simple_args(42, 4.2, TestStatus.Pending, True)
    
    # Test calling method with string arg
    rust_trait_obj.trait_fn_with_string_arg("Hello from trait object")
    
    # Test calling method with struct arg
    ts = TestStruct()
    ts.i32_field = 123
    rust_trait_obj.trait_fn_with_struct_arg(ts)
    
    # Test calling method with vec of primitives
    rust_trait_obj.trait_fn_with_vec_of_primitives([1, 2, 3, 4, 5])
    
    # Test calling method with vec of bools
    rust_trait_obj.trait_fn_with_vec_of_bools([True, False, True])
    
    # Test calling method with vec of enums
    rust_trait_obj.trait_fn_with_vec_of_enums([TestStatus.Active, TestStatus.Inactive])
    
    # Test calling method with vec of strings
    rust_trait_obj.trait_fn_with_vec_of_strings(["Hello", "Trait", "Object"])
    
    # Test calling method with vec of structs
    ts2_1 = TestStruct2()
    ts2_1.i32_field = 321
    ts2_2 = TestStruct2()
    ts2_2.i32_field = 654
    rust_trait_obj.trait_fn_with_vec_of_structs([ts2_1, ts2_2])
    
    # Test calling method with options
    ts2_opt = TestStruct2()
    ts2_opt.i32_field = 789
    rust_trait_obj.trait_fn_with_options(42, "Hello from trait object", False, TestStatus.Pending, ts2_opt)
    
    # Test return values from trait object methods
    assert rust_trait_obj.trait_fn_return_int() == 12345
    assert rust_trait_obj.trait_fn_return_string() == "String from trait object"
    assert rust_trait_obj.trait_fn_return_bool() == True
    assert rust_trait_obj.trait_fn_return_enum() == TestStatus.Inactive
    assert rust_trait_obj.trait_fn_return_struct().i32_field == 987
    
    # Test vec returns
    assert rust_trait_obj.trait_fn_return_vec_of_primitives() == [10, 20, 30]
    assert rust_trait_obj.trait_fn_return_vec_of_bools() == [True, False, True, True]
    assert rust_trait_obj.trait_fn_return_vec_of_strings() == ["Hello", "from", "trait", "object"]
    assert rust_trait_obj.trait_fn_return_vec_of_enums() == [TestStatus.Active, TestStatus.Inactive, TestStatus.Pending]
    
    vec_of_ts2 = rust_trait_obj.trait_fn_return_vec_of_structs()
    assert len(vec_of_ts2) == 2
    assert vec_of_ts2[0].i32_field == 111
    assert vec_of_ts2[1].i32_field == 222
    
    # Test option returns
    assert rust_trait_obj.trait_fn_returning_option_int(True) == 555
    assert rust_trait_obj.trait_fn_returning_option_string(True) == "Some string"
    assert rust_trait_obj.trait_fn_returning_option_bool(True) == False
    assert rust_trait_obj.trait_fn_returning_option_enum(True) == TestStatus.Pending
    assert rust_trait_obj.trait_fn_returning_option_struct(True).i32_field == 789

    assert rust_trait_obj.trait_fn_returning_option_int(False) is None
    assert rust_trait_obj.trait_fn_returning_option_string(False) is None
    assert rust_trait_obj.trait_fn_returning_option_bool(False) is None
    assert rust_trait_obj.trait_fn_returning_option_enum(False) is None
    assert rust_trait_obj.trait_fn_returning_option_struct(False) is None

    # Test that returned trait object can be passed to a function taking trait object
    function_taking_trait_object(rust_trait_obj)

    import gc

    gc.collect()


if __name__ == "__main__":
    struct_tests()
    functions_tests()
    methods_tests()
    static_methods_tests()
    vector_tests()
    vector_methods_tests()
    enum_tests()
    result_tests()
    option_tests()
    trait_tests()
    print("All tests passed!")
