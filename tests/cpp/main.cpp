#include <iostream>
#include "assertions.h"

int main()
{
    assert_structs();
    assert_default_impl();
    assert_struct_impl_block();
    assert_struct_methods_with_structs();
    assert_struct_methods_with_vectors();
    assert_functions();
    assert_vectors();
    assert_enums();
    assert_results();
    assert_options();
    assert_traits();
    assert_method_trait_objects();

    std::cout << "All assertions passed!" << std::endl;

    return 0;
}