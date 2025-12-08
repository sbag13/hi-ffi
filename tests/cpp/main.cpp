#include <iostream>
#include "assertions.h"

int main()
{
    assert_structs();
    assert_struct_impl_block();
    assert_struct_methods_with_structs();
    assert_functions();

    std::cout << "All assertions passed!" << std::endl;

    return 0;
}