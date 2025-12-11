#include "assertions.h"
#include "simple_function.h"
#include "function_with_primitive_and_string_arg.h"
#include "function_with_primitive_args.h"
#include "function_with_string_arg.h"
#include "function_return_primitive.h"
#include "function_return_string.h"
#include "function_taking_struct.h"
#include "function_returning_struct.h"
#include "combo_function.h"
#include "combo_struct_function.h"
#include "TestStruct.h"
#include <iostream>
#include <cassert>

void assert_structs()
{
    auto test_struct = TestStruct();

    assert(test_struct.get_i32_field() == 0);
    test_struct.set_i32_field(42);
    assert(test_struct.get_i32_field() == 42);

    assert(test_struct.get_f32_field() == 0.0f);
    test_struct.set_f32_field(3.14f);
    assert(test_struct.get_f32_field() == 3.14f);

    assert(test_struct.get_bool_field() == false);
    test_struct.set_bool_field(true);
    assert(test_struct.get_bool_field() == true);

    assert(test_struct.get_string_field() == "");
    test_struct.set_string_field("Hello, World!");
    assert(test_struct.get_string_field() == "Hello, World!");
    test_struct.set_string_field("Hello, Rust!");
    assert(test_struct.get_string_field() == "Hello, Rust!");
    std::string str = "Hello, C++!";
    test_struct.set_string_field(str);
    assert(test_struct.get_string_field() == "Hello, C++!");

    auto test_struct2 = test_struct.get_struct_field();
    test_struct2.set_i32_field(43);
    assert(test_struct2.get_i32_field() == 43);

    auto test_struct2_other = TestStruct2();
    test_struct2_other.set_i32_field(44);
    test_struct.set_struct_field(test_struct2_other);
    assert(test_struct.get_struct_field().get_i32_field() == 44);
}

void assert_struct_impl_block()
{
    auto test_struct = TestStruct();
    test_struct.public_method();
    TestStruct::static_method();
    test_struct.public_method_taking_primitives(5, true);
    TestStruct::static_method_taking_primitives(6, false);
    test_struct.public_method_taking_string("Hello, Rust!");
    TestStruct::static_method_taking_string("Hello, Rust!");
}

void assert_struct_methods_with_structs()
{
    auto test_struct = TestStruct();
    auto test_struct2_arg = TestStruct2();
    test_struct2_arg.set_i32_field(100);
    test_struct.public_method_taking_struct(test_struct2_arg);
    auto returned_struct = test_struct.public_method_returning_struct();
    assert(returned_struct.get_i32_field() == 99);
    auto static_arg = TestStruct2();
    static_arg.set_i32_field(200);
    TestStruct::static_method_taking_struct(static_arg);
    auto static_returned = TestStruct::static_method_returning_struct();
    assert(static_returned.get_i32_field() == 77);
    auto combo_arg1 = TestStruct2();
    combo_arg1.set_i32_field(300);
    auto combo_arg2 = TestStruct2();
    combo_arg2.set_i32_field(400);
    auto combo_result = TestStruct::static_combo_struct_method(combo_arg1, combo_arg2);
    assert(combo_result.get_i32_field() == 300);
}

void assert_functions()
{
    simple_function();
    function_with_primitive_args(3, true);
    function_with_string_arg("CPP string arg: Hello, World!");
    function_with_primitive_and_string_arg(42, false, "Complex function!");
    assert(function_return_primitive() == 42);
    assert(function_return_string() == "String returned from Rust");
    function_taking_struct(TestStruct2());
    auto struct2_from_function = function_returning_struct();
    assert(struct2_from_function.get_i32_field() == 48);
    assert(combo_function("str1", "str2", true, TestStruct()) == "str1");
    auto s1 = TestStruct();
    s1.set_i32_field(49);
    auto combo_struct_function_result = combo_struct_function(s1, TestStruct(), TestStruct2());
    assert(combo_struct_function_result.get_i32_field() == 49);
}
