#include "simple_function.h"
#include "function_with_primitive_and_string_arg.h"
#include "function_with_primitive_args.h"
#include "function_with_string_arg.h"
#include "function_return_primitive.h"
#include "function_return_string.h"
#include "combo_function.h"
#include "TestStruct.h"
#include <iostream>

// TODO assertions

int main()
{
    // Structs

    auto test_struct = TestStruct();

    std::cout << test_struct.get_i32_field() << std::endl;
    test_struct.set_i32_field(42);
    std::cout << test_struct.get_i32_field() << std::endl;

    std::cout << test_struct.get_bool_field() << std::endl;

    std::cout << "Empty string: " << test_struct.get_string_field() << std::endl;
    test_struct.set_string_field(std::string("Hello, World!"));
    std::cout << test_struct.get_string_field() << std::endl;
    test_struct.set_string_field("Hello, Rust!");
    std::cout << test_struct.get_string_field() << std::endl;
    std::string str = "Hello, C++!";
    test_struct.set_string_field(str);
    std::cout << test_struct.get_string_field() << std::endl;

    auto test_struct2 = test_struct.get_struct_field();
    test_struct2.set_i32_field(43);
    std::cout << test_struct2.get_i32_field() << std::endl;

    auto test_struct2_other = TestStruct2();
    test_struct2_other.set_i32_field(44);
    test_struct.set_struct_field(test_struct2_other);
    std::cout << "struct setter test: " << test_struct.get_struct_field().get_i32_field() << std::endl;

    // Functions
    simple_function();
    function_with_primitive_args(3, true);
    function_with_string_arg("CPP string arg: Hello, World!");
    function_with_primitive_and_string_arg(42, false, "Complex function!");
    std::cout << function_return_primitive() << std::endl;
    std::cout << function_return_string() << std::endl;
    std::cout << combo_function("Combo function!", "Don't print me", true) << std::endl;
}