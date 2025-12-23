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
#include "function_taking_vec_of_primitives.h"
#include "function_taking_vec_of_bools.h"
#include "function_taking_vec_of_strings.h"
#include "function_taking_vec_of_structs.h"
#include "function_returning_vec_of_int.h"
#include "function_returning_vec_of_bool.h"
#include "function_returning_vec_of_structs.h"
#include "function_returning_vec_of_string.h"
#include "function_taking_vec_of_enums.h"
#include "function_returning_vec_of_enums.h"
#include "TestStatus.h"
#include "get_status.h"
#include "get_inactive_status.h"
#include "assert_active.h"
#include "assert_inactive.h"
#include <iostream>
#include <cassert>

void assert_vectors()
{
    std::vector<i32> v = {1, 2, 3, 4, 5};
    function_taking_vec_of_primitives(v);

    std::vector<bool> v_bool = {true, false, true, true};
    function_taking_vec_of_bools(v_bool);

    std::vector<std::string> v_str = {"Hello, Rust!", "Hello, C++!", "Hello, Python!", "Hello, Swift!"};
    function_taking_vec_of_strings(v_str);

    auto s1 = TestStruct();
    s1.set_i32_field(15);
    auto s2 = TestStruct();
    s2.set_i32_field(17);
    std::vector<TestStruct> v_structs = {s1, s2};
    function_taking_vec_of_structs(v_structs);

    auto v_int = function_returning_vec_of_int();
    assert(v_int.size() == 4);
    std::vector<i32> expected_ints = {3, 2, 7, 8};
    assert(v_int == expected_ints);

    auto v_bools = function_returning_vec_of_bool();
    assert(v_bools.size() == 4);
    std::vector<bool> expected_bools = {true, false, true, true};
    assert(v_bools == expected_bools);

    auto returned_structs = function_returning_vec_of_structs();
    assert(returned_structs.size() == 2);
    assert(returned_structs[0].get_i32_field() == 8);
    assert(returned_structs[1].get_i32_field() == 11);

    auto returned_strings = function_returning_vec_of_string();
    assert(returned_strings.size() == 3);
    std::vector<std::string> expected_strings = {"Hello", "World", "Rust"};
    assert(returned_strings == expected_strings);

    // Test vector of enums
    std::vector<TestStatus> v_enums = {TestStatus::Active, TestStatus::Inactive, TestStatus::Pending, TestStatus::Active};
    function_taking_vec_of_enums(v_enums);

    auto returned_enums = function_returning_vec_of_enums();
    assert(returned_enums.size() == 4);
    std::vector<TestStatus> expected_enums = {TestStatus::Pending, TestStatus::Active, TestStatus::Inactive, TestStatus::Active};
    assert(returned_enums.size() == expected_enums.size());
    for (size_t i = 0; i < returned_enums.size(); ++i) {
        assert(returned_enums[i] == expected_enums[i]);
    }
}

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

void assert_struct_methods_with_vectors()
{
    std::cout << "assert_struct_methods_with_vectors" << std::endl;

    auto test_struct = TestStruct();
    
    // Test instance methods with vectors
    std::vector<i32> vec_primitives = {1, 2, 3, 4, 5};
    test_struct.public_method_taking_vec_of_primitives(vec_primitives);
    
    std::vector<std::string> vec_strings = {"Hello", "World"};
    test_struct.public_method_taking_vec_of_strings(vec_strings);
    
    std::vector<TestStruct2> vec_structs;
    auto s1 = TestStruct2();
    s1.set_i32_field(42);
    auto s2 = TestStruct2();
    s2.set_i32_field(24);
    vec_structs.push_back(s1);
    vec_structs.push_back(s2);
    test_struct.public_method_taking_vec_of_structs(vec_structs);
    
    // Test static methods with vectors
    std::vector<i32> static_vec_primitives = {6, 7, 8, 9, 10};
    TestStruct::static_method_taking_vec_of_primitives(static_vec_primitives);
    
    std::vector<std::string> static_vec_strings = {"Static", "Method"};
    TestStruct::static_method_taking_vec_of_strings(static_vec_strings);
    
    std::vector<TestStruct2> static_vec_structs;
    auto s3 = TestStruct2();
    s3.set_i32_field(100);
    auto s4 = TestStruct2();
    s4.set_i32_field(200);
    static_vec_structs.push_back(s3);
    static_vec_structs.push_back(s4);
    TestStruct::static_method_taking_vec_of_structs(static_vec_structs);
    
    // Test instance methods returning vectors
    auto returned_primitives = test_struct.public_method_returning_vec_of_primitives();
    std::vector<i32> expected_primitives = {10, 20, 30, 40, 50};
    assert(returned_primitives == expected_primitives);
    
    auto returned_strings = test_struct.public_method_returning_vec_of_strings();
    std::vector<std::string> expected_strings = {"Method", "Vector", "Return"};
    assert(returned_strings == expected_strings);
    
    auto returned_structs = test_struct.public_method_returning_vec_of_structs();
    assert(returned_structs.size() == 2);
    assert(returned_structs[0].get_i32_field() == 300);
    assert(returned_structs[1].get_i32_field() == 400);
    
    // Test static methods returning vectors
    auto static_returned_primitives = TestStruct::static_method_returning_vec_of_primitives();
    std::vector<i32> static_expected_primitives = {60, 70, 80, 90, 100};
    assert(static_returned_primitives == static_expected_primitives);
    
    auto static_returned_strings = TestStruct::static_method_returning_vec_of_strings();
    std::vector<std::string> static_expected_strings = {"Static", "Method", "Vector"};
    assert(static_returned_strings == static_expected_strings);
    
    auto static_returned_structs = TestStruct::static_method_returning_vec_of_structs();
    assert(static_returned_structs.size() == 2);
    assert(static_returned_structs[0].get_i32_field() == 500);
    assert(static_returned_structs[1].get_i32_field() == 600);
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

void assert_enums()
{
    std::cout << "assert_enums" << std::endl;
    assert(get_status() == TestStatus::Active);
    assert(get_inactive_status() == TestStatus::Inactive);

    assert_active(TestStatus::Active);
    assert_inactive(TestStatus::Inactive);
}

