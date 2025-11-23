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

int main()
{
    // Structs
    auto test_struct = TestStruct();

    std::cout << test_struct.get_i32_field() << std::endl;
    test_struct.set_i32_field(42);
    std::cout << test_struct.get_i32_field() << std::endl;
    assert(test_struct.get_i32_field() == 42);

    std::cout << test_struct.get_bool_field() << std::endl;
    assert(test_struct.get_bool_field() == false);

    std::cout << "Empty string: " << test_struct.get_string_field() << std::endl;
    assert(test_struct.get_string_field() == "");
    test_struct.set_string_field(std::string("Hello, World!"));
    std::cout << test_struct.get_string_field() << std::endl;
    assert(test_struct.get_string_field() == "Hello, World!");
    test_struct.set_string_field("Hello, Rust!");
    std::cout << test_struct.get_string_field() << std::endl;
    assert(test_struct.get_string_field() == "Hello, Rust!");
    std::string str = "Hello, C++!";
    test_struct.set_string_field(str);
    std::cout << test_struct.get_string_field() << std::endl;
    assert(test_struct.get_string_field() == "Hello, C++!");

    auto test_struct2 = test_struct.get_struct_field();
    test_struct2.set_i32_field(43);
    std::cout << test_struct2.get_i32_field() << std::endl;
    assert(test_struct2.get_i32_field() == 43);

    auto test_struct2_other = TestStruct2();
    test_struct2_other.set_i32_field(44);
    test_struct.set_struct_field(test_struct2_other);
    std::cout << "struct setter test: " << test_struct.get_struct_field().get_i32_field() << std::endl;
    assert(test_struct.get_struct_field().get_i32_field() == 44);

    // Struct impl block
    test_struct.public_method();
    TestStruct::static_method();
    test_struct.public_method_taking_primitives(5, true);
    TestStruct::static_method_taking_primitives(6, false);
    test_struct.public_method_taking_string(std::string("Hello, Rust!"));
    TestStruct::static_method_taking_string(std::string("Hello, Rust!"));
    
    // Struct methods with struct arguments and return types
    auto test_struct2_arg = TestStruct2();
    test_struct2_arg.set_i32_field(100);
    test_struct.public_method_taking_struct(test_struct2_arg);
    
    auto returned_struct = test_struct.public_method_returning_struct();
    std::cout << "Method returned struct with field: " << returned_struct.get_i32_field() << std::endl;
    assert(returned_struct.get_i32_field() == 99);
    
    // Static methods with struct arguments and return types
    auto static_arg = TestStruct2();
    static_arg.set_i32_field(200);
    TestStruct::static_method_taking_struct(static_arg);
    
    auto static_returned = TestStruct::static_method_returning_struct();
    std::cout << "Static method returned struct with field: " << static_returned.get_i32_field() << std::endl;
    assert(static_returned.get_i32_field() == 77);
    
    // Combo struct method
    auto combo_arg1 = TestStruct2();
    combo_arg1.set_i32_field(300);
    auto combo_arg2 = TestStruct2();
    combo_arg2.set_i32_field(400);
    auto combo_result = TestStruct::static_combo_struct_method(combo_arg1, combo_arg2);
    std::cout << "Combo struct method returned: " << combo_result.get_i32_field() << std::endl;
    assert(combo_result.get_i32_field() == 300);
    
    assert(test_struct.public_method_returning_primitive() == 24);
    assert(TestStruct::static_method_returning_primitive() == 22);
    std::cout << test_struct.public_method_returning_string() << std::endl;
    std::cout << TestStruct::static_method_returning_string() << std::endl;
    test_struct.combo_method("str1", "str2", true);
    TestStruct::static_combo_method("str1", "str2", true);

    // Functions
    simple_function();
    function_with_primitive_args(3, true);
    function_with_string_arg("CPP string arg: Hello, World!");
    function_with_primitive_and_string_arg(42, false, "Complex function!");
    std::cout << function_return_primitive() << std::endl;
    assert(function_return_primitive() == 42);
    std::cout << function_return_string() << std::endl;
    assert(function_return_string() == "String returned from Rust");
    std::cout << "Calling function_taking_struct" << std::endl;
    function_taking_struct(TestStruct2());
    auto struct2_from_function = function_returning_struct();
    std::cout << struct2_from_function.get_i32_field() << std::endl;
    assert(struct2_from_function.get_i32_field() == 48);
    assert(combo_function("str1", "str2", true, TestStruct()) == "str1");
    auto s1 = TestStruct();
    s1.set_i32_field(49);
    auto combo_struct_function_result = combo_struct_function(s1, TestStruct(), TestStruct2());
    assert(combo_struct_function_result.get_i32_field() == 49);
}