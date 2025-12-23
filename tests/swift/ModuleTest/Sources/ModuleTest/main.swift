import Foundation
import FfiModule

func assert_struct_basics() {
    let s = TestStruct()

    assert(s.bool_field == false, "Default bool should be false")
    s.bool_field = true
    assert(s.bool_field == true, "bool_field should be true after set")

    assert(s.i32_field == 0, "Default i32_field should be 0")
    s.i32_field = 42
    assert(s.i32_field == 42, "i32_field should be 42 after set")

    assert(s.f32_field == 0.0, "Default f32_field should be 0.0")
    s.f32_field = 3.14
    assert(s.f32_field == 3.14, "f32_field should be 3.14 after set")
    
    assert(s.string_field == "", "Default string_field should be empty")
    s.string_field = "Hello, World!"
    assert(s.string_field == "Hello, World!", "string_field should be updated")
    
    let new_struct_field = TestStruct2()
    new_struct_field.i32_field = 999
    s.struct_field = new_struct_field
    assert(s.struct_field.i32_field == 999, "struct_field.i32_field should be 999")
}

func assert_functions() {
    simple_function()
    function_with_primitive_args(1, true)
    function_with_string_arg("Hello, World!")
    function_with_primitive_and_string_arg(1, false, "Hello, World!")
    assert(function_return_primitive() == 42, "function_return_primitive should return 42")
    assert(function_return_string() == "String returned from Rust", "function_return_string should return correct string")
    let s = TestStruct()
    assert(combo_function("Combo!", "Don't print me", true, s) == "Combo!", "combo_function should return correct string")
    let s2 = function_returning_struct()
    assert(s2.i32_field == 48, "function_returning_struct should return struct with i32_field 48")
    function_taking_struct(s2)
    let combo_struct_result = combo_struct_function(s, TestStruct(), s2)
    assert(combo_struct_result.i32_field == s.i32_field, "combo_struct_result i32_field should match input")
}

func assert_struct_methods() {
    let s = TestStruct()
    s.public_method()
    s.public_method_taking_primitives(1, true)
    s.public_method_taking_string("Hello, World!")
    let struct_from_method = s.public_method_returning_struct()
    s.public_method_taking_struct(struct_from_method)
    assert(s.public_method_returning_primitive() == 24, "public_method_returning_primitive should return 24")
    assert(s.public_method_returning_string() == "String returned from Rust method", "public_method_returning_string should return correct string")
    assert(s.combo_method("Combo!", "Don't print me", true) == "Combo!", "combo_method should return correct string")
}

func assert_struct_static_methods() {
    let s2 = function_returning_struct()
    TestStruct.static_method()
    TestStruct.static_method_taking_primitives(1, true)
    TestStruct.static_method_taking_string("Hello, World!")
    assert(TestStruct.static_method_returning_primitive() == 22, "static_method_returning_primitive should return 22")
    assert(TestStruct.static_method_returning_string() == "String returned from Rust static method", "static_method_returning_string should return correct string")
    let static_struct = TestStruct.static_method_returning_struct()
    assert(static_struct.i32_field == 77, "static_method_returning_struct should return struct with i32_field 77")
    let static_combo_struct_result = TestStruct.static_combo_struct_method(s2, static_struct)
    assert(static_combo_struct_result.i32_field == s2.i32_field, "static_combo_struct_result i32_field should match input")
    assert(TestStruct.static_combo_method("Combo!", "Don't print me", true) == "Combo!", "static_combo_method should return correct string")
}

func assert_vec_functions() {
    let vec_ints: [i32] = [1, 2, 3, 4, 5]
    function_taking_vec_of_primitives(vec_ints)

    let vec_bools: [bool] = [true, false, true, true]
    function_taking_vec_of_bools(vec_bools)

    let vec_strings: [String] = ["Hello, Rust!", "Hello, C++!", "Hello, Python!", "Hello, Swift!"]
    function_taking_vec_of_strings(vec_strings)

    let s1 = TestStruct()
    s1.i32_field = 15
    let s2 = TestStruct()
    s2.i32_field = 17
    let vec_structs: [TestStruct] = [s1, s2]
    function_taking_vec_of_structs(vec_structs)
    
    let returned_vec_ints = function_returning_vec_of_int()
    assert(returned_vec_ints.count == 4)
    let expected_ints: [i32] = [3, 2, 7, 8]
    assert(returned_vec_ints == expected_ints)

    let returned_vec_bools = function_returning_vec_of_bool()
    assert(returned_vec_bools.count == 4)
    let expected_bools: [bool] = [true, false, true, true]
    assert(returned_vec_bools == expected_bools)

    let returned_vec_structs = function_returning_vec_of_structs()
    assert(returned_vec_structs.count == 2)
    assert(returned_vec_structs[0].i32_field == 8)
    assert(returned_vec_structs[1].i32_field == 11)

    let returned_vec_strings = function_returning_vec_of_string()
    assert(returned_vec_strings.count == 3)
    let expected_strings: [String] = ["Hello", "World", "Rust"]
    assert(returned_vec_strings == expected_strings)

    // Test vector of enums
    let vec_enums: [TestStatus] = [TestStatus.Active, TestStatus.Inactive, TestStatus.Pending, TestStatus.Active]
    function_taking_vec_of_enums(vec_enums)

    let returned_vec_enums = function_returning_vec_of_enums()
    assert(returned_vec_enums.count == 4)
    let expected_enums: [TestStatus] = [TestStatus.Pending, TestStatus.Active, TestStatus.Inactive, TestStatus.Active]
    assert(returned_vec_enums == expected_enums)
}

func assert_vec_methods() {
    print("assert_vec_methods")

    let testStruct = TestStruct()

    // Test instance methods with vector arguments
    let vecPrimitives: [i32] = [1, 2, 3, 4, 5]
    testStruct.public_method_taking_vec_of_primitives(vecPrimitives)

    let vecStrings: [String] = ["Hello", "World"]
    testStruct.public_method_taking_vec_of_strings(vecStrings)

    let s1 = TestStruct2()
    s1.i32_field = 42
    let s2 = TestStruct2()
    s2.i32_field = 24
    let vecStructs: [TestStruct2] = [s1, s2]
    testStruct.public_method_taking_vec_of_structs(vecStructs)

    // Test static methods with vector arguments
    let staticVecPrimitives: [i32] = [6, 7, 8, 9, 10]
    TestStruct.static_method_taking_vec_of_primitives(staticVecPrimitives)

    let staticVecStrings: [String] = ["Static", "Method"]
    TestStruct.static_method_taking_vec_of_strings(staticVecStrings)

    let s3 = TestStruct2()
    s3.i32_field = 100
    let s4 = TestStruct2()
    s4.i32_field = 200
    let staticVecStructs: [TestStruct2] = [s3, s4]
    TestStruct.static_method_taking_vec_of_structs(staticVecStructs)

    // Test instance methods returning vectors
    let returnedPrimitives = testStruct.public_method_returning_vec_of_primitives()
    let expectedPrimitives: [i32] = [10, 20, 30, 40, 50]
    assert(returnedPrimitives == expectedPrimitives, "Instance method returning primitives should match expected")

    let returnedStrings = testStruct.public_method_returning_vec_of_strings()
    let expectedStrings: [String] = ["Method", "Vector", "Return"]
    assert(returnedStrings == expectedStrings, "Instance method returning strings should match expected")

    let returnedStructs = testStruct.public_method_returning_vec_of_structs()
    assert(returnedStructs.count == 2, "Instance method should return 2 structs")
    assert(returnedStructs[0].i32_field == 300, "First struct should have i32_field 300")
    assert(returnedStructs[1].i32_field == 400, "Second struct should have i32_field 400")

    // Test static methods returning vectors
    let staticReturnedPrimitives = TestStruct.static_method_returning_vec_of_primitives()
    let staticExpectedPrimitives: [i32] = [60, 70, 80, 90, 100]
    assert(staticReturnedPrimitives == staticExpectedPrimitives, "Static method returning primitives should match expected")

    let staticReturnedStrings = TestStruct.static_method_returning_vec_of_strings()
    let staticExpectedStrings: [String] = ["Static", "Method", "Vector"]
    assert(staticReturnedStrings == staticExpectedStrings, "Static method returning strings should match expected")

    let staticReturnedStructs = TestStruct.static_method_returning_vec_of_structs()
    assert(staticReturnedStructs.count == 2, "Static method should return 2 structs")
    assert(staticReturnedStructs[0].i32_field == 500, "First static struct should have i32_field 500")
    assert(staticReturnedStructs[1].i32_field == 600, "Second static struct should have i32_field 600")
}

func assert_enums() {
    print("assert_enums")
    let status = get_status()
    assert(status == TestStatus.Active, "Enum should be Active")

    let status2 = get_inactive_status()
    assert(status2 == TestStatus.Inactive, "Enum should be Inactive")

    assert_active(TestStatus.Active)
    assert_inactive(TestStatus.Inactive)
}

func assert_struct_vector_fields() {
    print("assert_struct_vector_fields")

    let structWithVectors = StructWithVecs()

    // Test vec_of_ints
    let vecOfInts = structWithVectors.vec_of_ints
    assert(vecOfInts.count == 0, "vec_of_ints should be empty initially")
    let newVecOfInts: [i32] = [3, 5, 7]
    structWithVectors.vec_of_ints = newVecOfInts
    let vecOfInts2 = structWithVectors.vec_of_ints
    assert(vecOfInts2.count == 3, "vec_of_ints should have 3 elements after set")
    assert(vecOfInts2[0] == 3, "First element should be 3")
    assert(vecOfInts2[1] == 5, "Second element should be 5")
    assert(vecOfInts2[2] == 7, "Third element should be 7")

    // Test vec_of_bools
    let vecOfBools = structWithVectors.vec_of_bools
    assert(vecOfBools.count == 0, "vec_of_bools should be empty initially")
    let newVecOfBools: [bool] = [true, true, false]
    structWithVectors.vec_of_bools = newVecOfBools
    let vecOfBools2 = structWithVectors.vec_of_bools
    assert(vecOfBools2.count == 3, "vec_of_bools should have 3 elements after set")
    assert(vecOfBools2[0] == true, "First bool should be true")
    assert(vecOfBools2[1] == true, "Second bool should be true")
    assert(vecOfBools2[2] == false, "Third bool should be false")

    // Test vec_of_strings
    let vecOfStrings = structWithVectors.vec_of_strings
    assert(vecOfStrings.count == 0, "vec_of_strings should be empty initially")
    let newVecOfStrings: [String] = ["new", "vec"]
    structWithVectors.vec_of_strings = newVecOfStrings
    let vecOfStrings2 = structWithVectors.vec_of_strings
    assert(vecOfStrings2.count == 2, "vec_of_strings should have 2 elements after set")
    assert(vecOfStrings2[0] == "new", "First string should be 'new'")
    assert(vecOfStrings2[1] == "vec", "Second string should be 'vec'")

    // Test vec_of_structs
    let vecOfStructs = structWithVectors.vec_of_structs
    assert(vecOfStructs.count == 0, "vec_of_structs should be empty initially")
    let ts2 = TestStruct2()
    ts2.i32_field = 567
    let newVecOfStructs: [TestStruct2] = [TestStruct2(), ts2]
    structWithVectors.vec_of_structs = newVecOfStructs
    let vecOfStructs2 = structWithVectors.vec_of_structs
    assert(vecOfStructs2.count == 2, "vec_of_structs should have 2 elements after set")
    assert(vecOfStructs2[0].i32_field == 0, "First struct should have i32_field 0")
    assert(vecOfStructs2[1].i32_field == 567, "Second struct should have i32_field 567")
}

func run() {
    assert_struct_basics()
    assert_functions()
    assert_struct_methods()
    assert_struct_static_methods()
    assert_vec_functions()
    assert_vec_methods()
    assert_struct_vector_fields()
    assert_enums()
    print("All assertions passed.")
}

run()
