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
#include "StructWithVecs.h"
#include "function_with_primitive_result.h"
#include "function_with_bool_result.h"
#include "function_with_string_result.h"
#include "function_with_struct_result.h"
#include "function_with_enum_result.h"
#include "function_with_vec_int_result.h"
#include "function_with_vec_bool_result.h"
#include "function_with_vec_string_result.h"
#include "function_with_vec_struct_result.h"
#include "function_with_vec_enum_result.h"
#include "function_return_float.h"
#include "function_return_negated_bool.h"
#include "function_with_unit_expression_result.h"
#include "take_status_before_it_is_defined.h"
#include "take_struct_and_return_status_before_they_are_defined.h"
#include "function_taking_some_int.h"
#include "function_taking_some_bool.h"
#include "function_taking_some_string.h"
#include "function_taking_some_enum.h"
#include "function_taking_some_struct.h"
#include "function_returning_opt_int.h"
#include "function_returning_opt_bool.h"
#include "function_returning_opt_string.h"
#include "function_returning_opt_enum.h"
#include "function_returning_opt_struct.h"
#include "StructWithOptions.h"
#include "RustTrait.h"
#include "function_taking_trait_object.h"
#include <iostream>
#include <cassert>
#include <cstring>
#include <optional>

void assert_options()
{
    std::cout << "assert_options" << std::endl;

    auto some_i32 = std::optional<i32>(10);
    function_taking_some_int(some_i32, true);
    auto none_i32 = std::optional<i32>();
    function_taking_some_int(none_i32, false);

    auto some_bool = std::optional<bool>(true);
    function_taking_some_bool(some_bool, true);
    auto none_bool = std::optional<bool>();
    function_taking_some_bool(none_bool, false);

    auto some_string = std::optional<std::string>("Some string");
    function_taking_some_string(some_string, true);
    auto none_string = std::optional<std::string>();
    function_taking_some_string(none_string, false);

    auto some_enum = std::optional<TestStatus>(TestStatus::Pending);
    function_taking_some_enum(some_enum, true);
    auto none_enum = std::optional<TestStatus>();
    function_taking_some_enum(none_enum, false);

    auto ts1 = TestStruct();
    ts1.set_i32_field(567);
    auto some_struct = std::optional<TestStruct>(ts1);
    function_taking_some_struct(some_struct, true);

    auto ret_some_int = function_returning_opt_int(true);
    assert(ret_some_int == std::optional<i32>(100));
    auto ret_none_int = function_returning_opt_int(false);
    assert(ret_none_int == std::optional<i32>());

    auto ret_some_bool = function_returning_opt_bool(true);
    assert(ret_some_bool == std::optional<bool>(true));
    auto ret_none_bool = function_returning_opt_bool(false);
    assert(ret_none_bool == std::optional<bool>());

    auto ret_some_str = function_returning_opt_string(true);
    assert(ret_some_str == std::optional<std::string>("Some Rust String"));
    auto ret_none_str = function_returning_opt_string(false);
    assert(ret_none_str == std::optional<std::string>());

    auto ret_some_enum = function_returning_opt_enum(true);
    assert(ret_some_enum == std::optional<TestStatus>(TestStatus::Pending));
    auto ret_none_enum = function_returning_opt_enum(false);
    assert(ret_none_enum == std::optional<TestStatus>());

    auto ret_some_struct = function_returning_opt_struct(true);
    assert(ret_some_struct.value().get_i32_field() == 234);
    auto ret_none_struct = function_returning_opt_struct(false);
    assert(!ret_none_struct.has_value());

    // Methods
    auto test_struct = TestStruct();
    auto some_opt_int = std::optional<i32>(20);
    test_struct.method_taking_opt_int(some_opt_int, true);
    auto none_opt_int = std::optional<i32>();
    test_struct.method_taking_opt_int(none_opt_int, false);

    auto some_opt_string = std::optional<std::string>("Optional string");
    TestStruct::static_method_taking_opt_string(some_opt_string, true);
    auto none_opt_string = std::optional<std::string>();
    TestStruct::static_method_taking_opt_string(none_opt_string, false);

    auto some_opt_bool = std::optional<bool>(false);
    test_struct.method_taking_opt_bool(some_opt_bool, true);
    auto none_opt_bool = std::optional<bool>();
    test_struct.method_taking_opt_bool(none_opt_bool, false);

    auto some_opt_enum = std::optional<TestStatus>(TestStatus::Active);
    TestStruct::static_method_taking_opt_enum(some_opt_enum, true);
    auto none_opt_enum = std::optional<TestStatus>();
    TestStruct::static_method_taking_opt_enum(none_opt_enum, false);

    auto ts2 = TestStruct2();
    ts2.set_i32_field(789);
    auto some_opt_struct = std::optional<TestStruct2>(ts2);
    test_struct.method_taking_opt_struct(some_opt_struct, true);
    auto none_opt_struct = std::optional<TestStruct2>();
    test_struct.method_taking_opt_struct(none_opt_struct, false);

    auto some_opt_int_2 = test_struct.method_returning_opt_int(true);
    assert(some_opt_int_2 == std::optional<i32>(30));
    auto none_opt_int_2 = test_struct.method_returning_opt_int(false);
    assert(none_opt_int_2 == std::optional<i32>());

    auto some_opt_bool_2 = TestStruct::static_method_returning_opt_bool(true);
    assert(some_opt_bool_2 == std::optional<bool>(false));
    auto none_opt_bool_2 = TestStruct::static_method_returning_opt_bool(false);
    assert(none_opt_bool_2 == std::optional<bool>());

    auto some_opt_string_2 = test_struct.method_returning_opt_string(true);
    assert(some_opt_string_2 == std::optional<std::string>("Optional string from Rust"));
    auto none_opt_string_2 = test_struct.method_returning_opt_string(false);
    assert(none_opt_string_2 == std::optional<std::string>());

    auto some_opt_enum_2 = TestStruct::static_method_returning_opt_enum(true);
    assert(some_opt_enum_2 == std::optional<TestStatus>(TestStatus::Inactive));
    auto none_opt_enum_2 = TestStruct::static_method_returning_opt_enum(false);
    assert(none_opt_enum_2 == std::optional<TestStatus>());

    auto some_opt_struct_2 = test_struct.method_returning_opt_struct(true);
    assert(some_opt_struct_2.value().get_i32_field() == 654);
    auto none_opt_struct_2 = test_struct.method_returning_opt_struct(false);
    assert(!none_opt_struct_2.has_value());

    // Getters and setters
    auto options_struct = StructWithOptions();

    assert(!options_struct.get_opt_int().has_value());
    options_struct.set_opt_int(std::optional<i32>(555));
    assert(options_struct.get_opt_int().value() == 555);

    assert(!options_struct.get_opt_bool().has_value());
    options_struct.set_opt_bool(std::optional<bool>(true));
    assert(options_struct.get_opt_bool().value() == true);

    assert(!options_struct.get_opt_string().has_value());
    options_struct.set_opt_string(std::optional<std::string>("Opt string in struct"));
    assert(options_struct.get_opt_string().value() == "Opt string in struct");

    assert(!options_struct.get_opt_enum().has_value());
    options_struct.set_opt_enum(std::optional<TestStatus>(TestStatus::Pending));
    assert(options_struct.get_opt_enum().value() == TestStatus::Pending);

    auto ts3 = TestStruct2();
    ts3.set_i32_field(321);
    assert(!options_struct.get_opt_struct().has_value());
    options_struct.set_opt_struct(std::optional<TestStruct2>(ts3));
    assert(options_struct.get_opt_struct().value().get_i32_field() == 321);
}

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
    for (size_t i = 0; i < returned_enums.size(); ++i)
    {
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

    auto struct_with_vectors = StructWithVecs();

    auto vec_of_ints = struct_with_vectors.get_vec_of_ints();
    assert(vec_of_ints.size() == 0);
    std::vector<i32> new_vec_of_ints = {3, 5, 7};
    struct_with_vectors.set_vec_of_ints(new_vec_of_ints);
    auto vec_of_ints_2 = struct_with_vectors.get_vec_of_ints();
    assert(vec_of_ints_2.size() == 3);
    assert(vec_of_ints_2[0] == 3);
    assert(vec_of_ints_2[1] == 5);
    assert(vec_of_ints_2[2] == 7);

    auto vec_of_bools = struct_with_vectors.get_vec_of_bools();
    assert(vec_of_bools.size() == 0);
    std::vector<bool> new_vec_of_bools = {true, true, false};
    struct_with_vectors.set_vec_of_bools(new_vec_of_bools);
    auto vec_of_bools_2 = struct_with_vectors.get_vec_of_bools();
    assert(vec_of_bools_2.size() == 3);
    assert(vec_of_bools_2[0]);
    assert(vec_of_bools_2[1]);
    assert(!vec_of_bools_2[2]);

    auto vec_of_strings = struct_with_vectors.get_vec_of_strings();
    assert(vec_of_strings.size() == 0);
    std::vector<std::string> new_vec_of_strings = {"new", "vec"};
    struct_with_vectors.set_vec_of_strings(new_vec_of_strings);
    auto vec_of_strings_2 = struct_with_vectors.get_vec_of_strings();
    assert(vec_of_strings_2.size() == 2);
    assert(vec_of_strings_2[0] == "new");
    assert(vec_of_strings_2[1] == "vec");

    auto vec_of_structs = struct_with_vectors.get_vec_of_structs();
    assert(vec_of_structs.size() == 0);
    auto ts2 = TestStruct2();
    ts2.set_i32_field(567);
    std::vector<TestStruct2> new_vec_of_structs = {TestStruct2(), ts2};
    struct_with_vectors.set_vec_of_structs(new_vec_of_structs);
    auto vec_of_structs_2 = struct_with_vectors.get_vec_of_structs();
    assert(vec_of_structs_2.size() == 2);
    assert(vec_of_structs_2[0].get_i32_field() == 0);
    assert(vec_of_structs_2[1].get_i32_field() == 567);

    auto vec_of_enums = struct_with_vectors.get_vec_of_enums();
    assert(vec_of_enums.size() == 0);
    std::vector<TestStatus> new_vec_of_enums = {TestStatus::Pending, TestStatus::Active, TestStatus::Inactive};
    struct_with_vectors.set_vec_of_enums(new_vec_of_enums);
    auto vec_of_enums_2 = struct_with_vectors.get_vec_of_enums();
    assert(vec_of_enums_2.size() == 3);
    assert(vec_of_enums_2[0] == TestStatus::Pending);
    assert(vec_of_enums_2[1] == TestStatus::Active);
    assert(vec_of_enums_2[2] == TestStatus::Inactive);
}

void assert_struct_impl_block()
{
    auto test_struct = TestStruct();
    test_struct.public_method();
    TestStruct::static_method();
    test_struct.public_method_taking_primitives(10, false);
    TestStruct::static_method_taking_primitives(20, true);
    assert(test_struct.public_method_returning_primitive() == 24);
    assert(test_struct.public_method_returning_bool() == true);
    assert(test_struct.public_method_returning_string() == "String returned from Rust method");
    assert(test_struct.combo_method("str1", "str2", true) == "str1");
    test_struct.public_method_taking_string("Test string from caller");
    TestStruct::static_method_taking_string("static method string");
    assert(TestStruct::static_method_returning_primitive() == 22);
    assert(TestStruct::static_method_returning_string() == "String returned from Rust static method");
    assert(test_struct.combo_method("first", "second", false) == "second");
}

void assert_struct_methods_with_structs()
{
    auto test_struct = TestStruct();
    auto test_struct2_arg = TestStruct2();
    test_struct2_arg.set_i32_field(55);
    test_struct.public_method_taking_struct(test_struct2_arg);
    auto returned_struct = test_struct.public_method_returning_struct();
    assert(returned_struct.get_i32_field() == 99);
    auto static_arg = TestStruct2();
    static_arg.set_i32_field(77);
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
    std::vector<u16> static_vec_primitives = {6, 7, 8, 9, 10};
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
    take_status_before_it_is_defined(TestStatus::Pending);
    auto ts1 = TestStruct();
    ts1.set_i32_field(-5);
    assert(take_struct_and_return_status_before_they_are_defined(ts1) == TestStatus::Pending);
    simple_function();
    function_with_primitive_args(100, true);
    function_with_string_arg("Hello, World!");
    function_with_primitive_and_string_arg(42, false, "Complex function!");
    assert(function_return_primitive() == 42);
    assert(function_return_float() == 5.21);
    assert(function_return_string() == "String returned from Rust");
    assert(function_return_negated_bool(true) == false);
    auto ts2 = TestStruct2();
    ts2.set_i32_field(55);
    function_taking_struct(ts2);
    auto struct2_from_function = function_returning_struct();
    assert(struct2_from_function.get_i32_field() == 48);
    auto new_ts1 = TestStruct();
    assert(combo_function("str1", "str2", true, new_ts1) == "str1");
    auto s1 = TestStruct();
    s1.set_i32_field(49);
    auto new_ts2 = TestStruct2();
    auto combo_struct_function_result = combo_struct_function(s1, new_ts1, new_ts2);
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

void assert_results()
{
    std::cout << "assert_results" << std::endl;

    // Test function that returns Result<i32, Error>
    assert(function_with_primitive_result(false) == 123);
    try
    {
        function_with_primitive_result(true);
        assert(false); // Should not reach here
    }
    catch (const RustException &e)
    {
        assert(strcmp(e.what(), "StructError: EnumError: VariantTwo") == 0);
        auto source_err = e.source().value();
        assert(strcmp(source_err.what(), "EnumError: VariantTwo") == 0);
        auto source_of_source = source_err.source().value();
        assert(strcmp(source_of_source.what(), "SimpleError") == 0);
        auto source_of_source_2 = source_of_source.source();
        assert(!source_of_source_2.has_value());
    }

    function_with_unit_expression_result(); // just no exception
    assert(function_with_bool_result(false) == true);
    assert(function_with_string_result(false) == "No error");
    assert(function_with_struct_result(false).get_i32_field() == 256);
    assert(function_with_enum_result(false) == TestStatus::Pending);
    assert(function_with_vec_int_result(false) == std::vector<i32>({10, 20, 30}));
    assert(function_with_vec_bool_result(false) == std::vector<bool>({true, false, false}));
    assert(function_with_vec_string_result(false) == std::vector<std::string>({"One", "Two", "Three"}));
    auto vec_of_structs = function_with_vec_struct_result(false);
    assert(vec_of_structs.size() == 2);
    assert(vec_of_structs[0].get_i32_field() == 512);
    assert(vec_of_structs[1].get_i32_field() == 1024);
    auto vec_of_enums = function_with_vec_enum_result(false);
    assert(vec_of_enums.size() == 3);
    assert(vec_of_enums[0] == TestStatus::Pending);
    assert(vec_of_enums[1] == TestStatus::Active);
    assert(vec_of_enums[2] == TestStatus::Inactive);

    auto ts1 = TestStruct();
    try
    {
        ts1.method_with_int_result(true);
        assert(false);
    }
    catch (const RustException &e)
    {
        assert(strcmp(e.what(), "StructError: EnumError: VariantTwo") == 0);
    }

    assert(ts1.method_with_int_result(false) == 16);
    assert(TestStruct::static_method_with_bool_result(false));
    assert(ts1.method_with_string_result() == "Ok!");
    assert(TestStruct::static_method_with_struct_result().get_i32_field() == 267);
    assert(ts1.method_with_enum_result() == TestStatus::Pending);
    assert(ts1.method_with_vec_of_ints_result() == std::vector<i8>({1, 2, 7}));
    assert(TestStruct::static_method_with_vec_of_bools_result() == std::vector<bool>({true, false, false}));
    assert(ts1.method_with_vec_of_strings_result() == std::vector<std::string>({"some", "string"}));
    auto ok_vec_of_structs = TestStruct::static_method_with_vec_of_structs_result();
    assert(ok_vec_of_structs.size() == 2);
    assert(ok_vec_of_structs[0].get_i32_field() == 2);
    assert(ok_vec_of_structs[1].get_i32_field() == -5);
    assert(ts1.method_with_vec_of_bools_result() == std::vector<bool>({false, true, true, true}));
    assert(TestStruct::static_method_with_vec_of_enum_result() == std::vector<TestStatus>({TestStatus::Inactive, TestStatus::Pending, TestStatus::Active}));
}

class MyStructWithTrait : public RustTrait
{
public:
    void trait_simple_fn() override
    {
        // std::cout << "MyStructWithTrait::trait_simple_fn called!" << std::endl;
    }

    void trait_fn_with_simple_args(i32 i, f32 f, TestStatus e, bool b) override
    {
        assert(i == 42);
        assert(f == 4.2f);
        assert(e == TestStatus::Pending);
        assert(b == true);
    }

    void trait_fn_with_string_arg(std::string s) override
    {
        assert(s == "Hello from trait object");
    }

    void trait_fn_with_struct_arg(TestStruct &s) override
    {
        assert(s.get_i32_field() == 123);
    }

    void trait_fn_with_vec_of_primitives(std::vector<i32> &vec) override
    {
        std::vector<i32> expected = {1, 2, 3, 4, 5};
        assert(vec == expected);
    }

    void trait_fn_with_vec_of_bools(std::vector<bool> &vec) override
    {
        std::vector<bool> expected = {true, false, true};
        assert(vec == expected);
    }

    void trait_fn_with_vec_of_strings(std::vector<std::string> &vec) override
    {
        std::vector<std::string> expected = {"Hello", "Trait", "Object"};
        assert(vec == expected);
    }

    void trait_fn_with_vec_of_structs(std::vector<TestStruct2> &vec) override
    {
        assert(vec.size() == 2);
        assert(vec[0].get_i32_field() == 321);
        assert(vec[1].get_i32_field() == 654);
    }

    void trait_fn_with_vec_of_enums(std::vector<TestStatus> &vec) override
    {
        std::vector<TestStatus> expected = {TestStatus::Active, TestStatus::Inactive};
        assert(vec == expected);
    }

    void trait_fn_with_options(std::optional<i64> &opt_int, std::optional<std::string> &opt_string, std::optional<bool> &opt_bool, std::optional<TestStatus> &opt_enum, std::optional<TestStruct2> &opt_struct) override
    {
        assert(opt_int.value() == 42);
        assert(opt_string.value() == "Hello from trait object");
        assert(opt_bool.value() == false);
        assert(opt_enum.value() == TestStatus::Pending);
        assert(opt_struct.value().get_i32_field() == 789);
    }

    i32 trait_fn_return_int() override
    {
        return 12345;
    }

    bool trait_fn_return_bool() override
    {
        return true;
    }

    std::string trait_fn_return_string() override
    {
        return "String from trait object";
    }

    TestStruct2 trait_fn_return_struct() override
    {
        TestStruct2 s;
        s.set_i32_field(987);
        return s;
    }

    TestStatus trait_fn_return_enum() override
    {
        return TestStatus::Inactive;
    }

    std::vector<i32> trait_fn_return_vec_of_primitives() override
    {
        return {10, 20, 30};
    }

    std::vector<bool> trait_fn_return_vec_of_bools() override
    {
        return {true, false, true, true};
    }

    std::vector<std::string> trait_fn_return_vec_of_strings() override
    {
        return {"Hello", "from", "trait", "object"};
    }

    std::vector<TestStatus> trait_fn_return_vec_of_enums() override
    {
        return {TestStatus::Active, TestStatus::Inactive, TestStatus::Pending};
    }

    std::vector<TestStruct2> trait_fn_return_vec_of_structs() override
    {
        TestStruct2 s1;
        s1.set_i32_field(111);
        TestStruct2 s2;
        s2.set_i32_field(222);
        return {s1, s2};
    }

    std::optional<i32> trait_fn_returning_option_int(bool some) override
    {
        if (some)
        {
            return std::optional<i32>(555);
        }
        else
        {
            return std::optional<i32>();
        }
    }

    std::optional<std::string> trait_fn_returning_option_string(bool some) override
    {
        if (some)
        {
            return std::optional<std::string>("Some string");
        }
        else
        {
            return std::optional<std::string>();
        }
    }

    std::optional<bool> trait_fn_returning_option_bool(bool some) override
    {
        if (some)
        {
            return std::optional<bool>(false);
        }
        else
        {
            return std::optional<bool>();
        }
    }

    std::optional<TestStatus> trait_fn_returning_option_enum(bool some) override
    {
        if (some)
        {
            return std::optional<TestStatus>(TestStatus::Pending);
        }
        else
        {
            return std::optional<TestStatus>();
        }
    }

    std::optional<TestStruct2> trait_fn_returning_option_struct(bool some) override
    {
        if (some)
        {
            TestStruct2 s;
            s.set_i32_field(789);
            return std::optional<TestStruct2>(s);
        }
        else
        {
            return std::optional<TestStruct2>();
        }
    }

    ~MyStructWithTrait() override
    {
        // std::cout << "MyStructWithTrait destructor called!" << std::endl;
    }
};

void assert_traits()
{
    std::cout << "assert_traits" << std::endl;
    function_taking_trait_object(std::make_unique<MyStructWithTrait>());
}
