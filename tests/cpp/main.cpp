#include <iostream>
#include "assertions.h"

int main()
{
    assert_structs();
    assert_struct_impl_block();
    assert_struct_methods_with_structs();
    assert_struct_methods_with_vectors();
    assert_functions();
    assert_vectors();
    assert_enums();

    std::cout << "All assertions passed!" << std::endl;

    return 0;
}