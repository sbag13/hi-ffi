import Foundation
import FfiModule

func assert_struct_basics() {
    let s = TestStruct()

    assert(s.i32_field == 0)
    s.i32_field = 42
    assert(s.i32_field == 42)

    assert(s.f32_field == 0.0)
    s.f32_field = 3.14
    assert(s.f32_field == 3.14)

    assert(s.bool_field == false)
    s.bool_field = true
    assert(s.bool_field == true)

    assert(s.string_field == "")
    s.string_field = "Hello, World!"
    assert(s.string_field == "Hello, World!")
    s.string_field = "Hello, Rust!"
    assert(s.string_field == "Hello, Rust!")
    s.string_field = "Hello, C++!"
    assert(s.string_field == "Hello, C++!")

    let test_struct2 = s.struct_field
    test_struct2.i32_field = 43
    assert(test_struct2.i32_field == 43)

    let test_struct2_other = TestStruct2()
    test_struct2_other.i32_field = 44
    s.struct_field = test_struct2_other
    assert(s.struct_field.i32_field == 44)

    let struct_with_vectors = StructWithVecs()

    let vec_of_ints = struct_with_vectors.vec_of_ints
    assert(vec_of_ints.count == 0)
    let new_vec_of_ints: [i32] = [3, 5, 7]
    struct_with_vectors.vec_of_ints = new_vec_of_ints
    let vec_of_ints_2 = struct_with_vectors.vec_of_ints
    assert(vec_of_ints_2.count == 3)
    assert(vec_of_ints_2[0] == 3)
    assert(vec_of_ints_2[1] == 5)
    assert(vec_of_ints_2[2] == 7)

    let vec_of_bools = struct_with_vectors.vec_of_bools
    assert(vec_of_bools.count == 0)
    let new_vec_of_bools: [Bool] = [true, true, false]
    struct_with_vectors.vec_of_bools = new_vec_of_bools
    let vec_of_bools_2 = struct_with_vectors.vec_of_bools
    assert(vec_of_bools_2.count == 3)
    assert(vec_of_bools_2[0])
    assert(vec_of_bools_2[1])
    assert(!vec_of_bools_2[2])

    let vec_of_strings = struct_with_vectors.vec_of_strings
    assert(vec_of_strings.count == 0)
    let new_vec_of_strings: [String] = ["new", "vec"]
    struct_with_vectors.vec_of_strings = new_vec_of_strings
    let vec_of_strings_2 = struct_with_vectors.vec_of_strings
    assert(vec_of_strings_2.count == 2)
    assert(vec_of_strings_2[0] == "new")
    assert(vec_of_strings_2[1] == "vec")

    let vec_of_structs = struct_with_vectors.vec_of_structs
    assert(vec_of_structs.count == 0)
    let ts2 = TestStruct2()
    ts2.i32_field = 567
    let new_vec_of_structs: [TestStruct2] = [TestStruct2(), ts2]
    struct_with_vectors.vec_of_structs = new_vec_of_structs
    let vec_of_structs_2 = struct_with_vectors.vec_of_structs
    assert(vec_of_structs_2.count == 2)
    assert(vec_of_structs_2[0].i32_field == 0)
    assert(vec_of_structs_2[1].i32_field == 567)

    let vec_of_enums = struct_with_vectors.vec_of_enums
    assert(vec_of_enums.count == 0)
    let new_vec_of_enums: [TestStatus] = [TestStatus.Pending, TestStatus.Active, TestStatus.Inactive]
    struct_with_vectors.vec_of_enums = new_vec_of_enums
    let vec_of_enums_2 = struct_with_vectors.vec_of_enums
    assert(vec_of_enums_2.count == 3)
    assert(vec_of_enums_2[0] == TestStatus.Pending)
    assert(vec_of_enums_2[1] == TestStatus.Active)
    assert(vec_of_enums_2[2] == TestStatus.Inactive)
}

func assert_functions() {
    take_status_before_it_is_defined(TestStatus.Pending)
    let ts1 = TestStruct()
    ts1.i32_field = (-5)
    assert(take_struct_and_return_status_before_they_are_defined(ts1) == TestStatus.Pending);
    simple_function()
    function_with_primitive_args(100, true)
    function_with_string_arg("Hello, World!")
    function_with_primitive_and_string_arg(42, false, "Complex function!")
    assert(function_return_primitive() == 42, "function_return_primitive should return 42")
    assert(function_return_float() == 5.21, "function_return_float should return 5.21")
    assert(function_return_string() == "String returned from Rust", "function_return_string should return correct string")
    assert(function_return_negated_bool(true) == false, "function_return_negated_bool should return false when input is true")
    let s = TestStruct()
    assert(combo_function("Combo!", "Don't print me", true, s) == "Combo!", "combo_function should return correct string")
    let s2 = function_returning_struct()
    assert(s2.i32_field == 48, "function_returning_struct should return struct with i32_field 48")
    s2.i32_field = 55
    function_taking_struct(s2)
    let combo_struct_result = combo_struct_function(s, TestStruct(), s2)
    assert(combo_struct_result.i32_field == s.i32_field, "combo_struct_result i32_field should match input")
}

func assert_struct_methods() {
    let s = TestStruct()
    s.public_method()
    s.public_method_taking_primitives(10, false)
    s.public_method_taking_string("Test string from caller")
    let struct_from_method = s.public_method_returning_struct()
    struct_from_method.i32_field = 55
    s.public_method_taking_struct(struct_from_method)
    assert(s.public_method_returning_primitive() == 24, "public_method_returning_primitive should return 24")
    assert(s.public_method_returning_string() == "String returned from Rust method", "public_method_returning_string should return correct string")
    assert(s.combo_method("Combo!", "Don't print me", true) == "Combo!", "combo_method should return correct string")
}

func assert_struct_static_methods() {
    let s2 = function_returning_struct()
    TestStruct.static_method()
    TestStruct.static_method_taking_primitives(20, true)
    TestStruct.static_method_taking_string("static method string")
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
    let staticVecPrimitives: [u16] = [6, 7, 8, 9, 10]
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

func assert_results() {
    print("assert_results")

    do {
        let i32_ok = try function_with_primitive_result(false)
        assert(i32_ok == 123, "function_with_primitive_result should return 123 on success")
    } catch {
        assert(false, "function_with_primitive_result should not throw on success")
    }

    do {
        let _ = try function_with_primitive_result(true)
        assert(false)
    } catch let rustError as RustError {
        assert(rustError.description() == "StructError: EnumError: VariantTwo")
        let source = rustError.source()
        assert(source?.description() == "EnumError: VariantTwo")
        let source2 = source?.source()
        assert(source2?.description() == "SimpleError")
        assert(source2?.source() == nil)
    } catch {
        assert(false, "function_with_primitive_result threw unexpected error type")
    }

    do {
        let bool_ok = try function_with_bool_result(false)
        assert(bool_ok)
        let string_ok = try function_with_string_result(false)
        assert(string_ok == "No error")
        let struct_ok = try function_with_struct_result(false)
        assert(struct_ok.i32_field == 256)
        let status_ok = try function_with_enum_result(false)
        assert(status_ok == TestStatus.Pending)
        let vec_int_ok = try function_with_vec_int_result(false)
        let expected_vec: [i32] = [10, 20, 30]
        assert(vec_int_ok == expected_vec)
        let vec_bool_ok = try function_with_vec_bool_result(false)
        let expected_vec_bool: [bool] = [true, false, false]
        assert(vec_bool_ok == expected_vec_bool)
        let vec_string_ok = try function_with_vec_string_result(false)
        let expected_vec_string: [String] = ["One", "Two", "Three"]
        assert(vec_string_ok == expected_vec_string)
        let vec_struct_ok = try function_with_vec_struct_result(false)
        assert(vec_struct_ok.count == 2)
        assert(vec_struct_ok[0].i32_field == 512)
        assert(vec_struct_ok[1].i32_field == 1024)
        let vec_enum_ok = try function_with_vec_enum_result(false)
        let expected_vec_enum: [TestStatus] = [TestStatus.Pending, TestStatus.Active, TestStatus.Inactive]
        assert(vec_enum_ok == expected_vec_enum)
        try function_with_unit_expression_result()
    } catch {
        assert(false)
    }

    // methods
    let ts1 = TestStruct()
    do {
        let _ = try ts1.method_with_int_result(true);
        assert(false)
    } catch let rustError as RustError {
        assert(rustError.description() == "StructError: EnumError: VariantTwo")
    } catch {
        assert(false)
    }

    do {
        let ok_int = try ts1.method_with_int_result(false)
        assert(ok_int == 16);
        let ok_bool = try TestStruct.static_method_with_bool_result(false)
        assert(ok_bool)
        let ok_str = try ts1.method_with_string_result()
        assert(ok_str == "Ok!")
        let ok_struct = try TestStruct.static_method_with_struct_result()
        assert(ok_struct.i32_field == 267)
        let ok_enum = try ts1.method_with_enum_result()
        assert(ok_enum == TestStatus.Pending)
        let ok_vec_of_ints = try ts1.method_with_vec_of_ints_result()
        assert(ok_vec_of_ints == [1, 2, 7])
        let ok_vec_of_bools = try TestStruct.static_method_with_vec_of_bools_result()
        assert(ok_vec_of_bools == [true, false, false])
        let ok_vec_of_bools_2 = try ts1.method_with_vec_of_bools_result()
        assert(ok_vec_of_bools_2 == [false, true, true, true])
        let ok_vec_of_strings = try ts1.method_with_vec_of_strings_result()
        assert(ok_vec_of_strings == ["some", "string"])
        let ok_vec_of_structs = try TestStruct.static_method_with_vec_of_structs_result()
        assert(ok_vec_of_structs.count == 2)
        assert(ok_vec_of_structs[0].i32_field == 2);
        assert(ok_vec_of_structs[1].i32_field == -5);
        let ok_vec_enum = try TestStruct.static_method_with_vec_of_enum_result()
        assert(ok_vec_enum == [TestStatus.Inactive, TestStatus.Pending, TestStatus.Active])
    } catch {
        assert(false)
    }
}

func assert_options() {
    print("assert_options")
    
    // Test function arguments with Option types
    let some_i32: i32? = 10
    function_taking_some_int(some_i32, true)
    let none_i32: i32? = nil
    function_taking_some_int(none_i32, false)
    
    let some_bool: Bool? = true
    function_taking_some_bool(some_bool, true)
    let none_bool: Bool? = nil
    function_taking_some_bool(none_bool, false)
    
    let some_string: String? = "Some string"
    function_taking_some_string(some_string, true)
    let none_string: String? = nil
    function_taking_some_string(none_string, false)
    
    let some_enum: TestStatus? = TestStatus.Pending
    function_taking_some_enum(some_enum, true)
    let none_enum: TestStatus? = nil
    function_taking_some_enum(none_enum, false)
    
    let ts1 = TestStruct()
    ts1.i32_field = 567
    let some_struct: TestStruct? = ts1
    function_taking_some_struct(some_struct, true)
    let none_struct: TestStruct? = nil
    function_taking_some_struct(none_struct, false)
    
    // Test function return values with Option types
    let ret_some_int = function_returning_opt_int(true)
    assert(ret_some_int == 100, "function_returning_opt_int(true) should return 100")
    let ret_none_int = function_returning_opt_int(false)
    assert(ret_none_int == nil, "function_returning_opt_int(false) should return nil")
    
    let ret_some_bool = function_returning_opt_bool(true)
    assert(ret_some_bool == true, "function_returning_opt_bool(true) should return true")
    let ret_none_bool = function_returning_opt_bool(false)
    assert(ret_none_bool == nil, "function_returning_opt_bool(false) should return nil")
    
    let ret_some_str = function_returning_opt_string(true)
    assert(ret_some_str == "Some Rust String", "function_returning_opt_string(true) should return correct string")
    let ret_none_str = function_returning_opt_string(false)
    assert(ret_none_str == nil, "function_returning_opt_string(false) should return nil")
    
    let ret_some_enum = function_returning_opt_enum(true)
    assert(ret_some_enum == TestStatus.Pending, "function_returning_opt_enum(true) should return Pending")
    let ret_none_enum = function_returning_opt_enum(false)
    assert(ret_none_enum == nil, "function_returning_opt_enum(false) should return nil")
    
    let ret_some_struct = function_returning_opt_struct(true)
    assert(ret_some_struct?.i32_field == 234, "function_returning_opt_struct(true) should return struct with i32_field 234")
    let ret_none_struct = function_returning_opt_struct(false)
    assert(ret_none_struct == nil, "function_returning_opt_struct(false) should return nil")
    
    // Test struct methods with Option types
    let test_struct = TestStruct()
    
    let some_opt_int: i32? = 20
    test_struct.method_taking_opt_int(some_opt_int, true)
    let none_opt_int: i32? = nil
    test_struct.method_taking_opt_int(none_opt_int, false)
    
    let some_opt_string: String? = "Optional string"
    TestStruct.static_method_taking_opt_string(some_opt_string, true)
    let none_opt_string: String? = nil
    TestStruct.static_method_taking_opt_string(none_opt_string, false)
    
    let some_opt_bool: Bool? = false
    test_struct.method_taking_opt_bool(some_opt_bool, true)
    let none_opt_bool: Bool? = nil
    test_struct.method_taking_opt_bool(none_opt_bool, false)
    
    let some_opt_enum: TestStatus? = TestStatus.Active
    TestStruct.static_method_taking_opt_enum(some_opt_enum, true)
    let none_opt_enum: TestStatus? = nil
    TestStruct.static_method_taking_opt_enum(none_opt_enum, false)
    
    let ts2 = TestStruct2()
    ts2.i32_field = 789
    let some_opt_struct: TestStruct2? = ts2
    test_struct.method_taking_opt_struct(some_opt_struct, true)
    let none_opt_struct: TestStruct2? = nil
    test_struct.method_taking_opt_struct(none_opt_struct, false)
    
    // Test struct methods returning Option types
    let some_opt_int_2 = test_struct.method_returning_opt_int(true)
    assert(some_opt_int_2 == 30, "method_returning_opt_int(true) should return 30")
    let none_opt_int_2 = test_struct.method_returning_opt_int(false)
    assert(none_opt_int_2 == nil, "method_returning_opt_int(false) should return nil")
    
    let some_opt_bool_2 = TestStruct.static_method_returning_opt_bool(true)
    assert(some_opt_bool_2 == false, "static_method_returning_opt_bool(true) should return false")
    let none_opt_bool_2 = TestStruct.static_method_returning_opt_bool(false)
    assert(none_opt_bool_2 == nil, "static_method_returning_opt_bool(false) should return nil")
    
    let some_opt_str = test_struct.method_returning_opt_string(true)
    assert(some_opt_str == "Optional string from Rust", "method_returning_opt_string(true) should return correct string")
    let none_opt_str = test_struct.method_returning_opt_string(false)
    assert(none_opt_str == nil, "method_returning_opt_string(false) should return nil")
    
    let some_opt_enum_2 = TestStruct.static_method_returning_opt_enum(true)
    assert(some_opt_enum_2 == TestStatus.Inactive, "static_method_returning_opt_enum(true) should return Active")
    let none_opt_enum_2 = TestStruct.static_method_returning_opt_enum(false)
    assert(none_opt_enum_2 == nil, "static_method_returning_opt_enum(false) should return nil")
    
    let some_opt_struct_2 = test_struct.method_returning_opt_struct(true)
    assert(some_opt_struct_2?.i32_field == 654, "method_returning_opt_struct(true) should return struct with i32_field 345")
    let none_opt_struct_2 = test_struct.method_returning_opt_struct(false)
    assert(none_opt_struct_2 == nil, "method_returning_opt_struct(false) should return nil")

    // Test struct fields with Option types
    let structWithOptions = StructWithOptions()

    assert(structWithOptions.opt_int == nil, "opt_i32_field should be nil initially")
    structWithOptions.opt_int = 123
    assert(structWithOptions.opt_int == 123, "opt_i32_field should be 123 after set")

    assert(structWithOptions.opt_bool == nil, "opt_bool_field should be nil initially")
    structWithOptions.opt_bool = true
    assert(structWithOptions.opt_bool == true, "opt_bool_field should be true after set")

    assert(structWithOptions.opt_string == nil, "opt_string_field should be nil initially")
    structWithOptions.opt_string = "Optional string field"
    assert(structWithOptions.opt_string == "Optional string field", "opt_string_field should be correct after set")

    assert(structWithOptions.opt_enum == nil, "opt_enum_field should be nil initially")
    structWithOptions.opt_enum = TestStatus.Active
    assert(structWithOptions.opt_enum == TestStatus.Active, "opt_enum_field should be Active after set")

    assert(structWithOptions.opt_struct == nil, "opt_struct_field should be nil initially")
    let ts3 = TestStruct2()
    ts3.i32_field = 321
    structWithOptions.opt_struct = ts3
    assert(structWithOptions.opt_struct?.i32_field == 321, "opt_struct_field should have i32_field 321 after set")
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
    assert_results()
    assert_options()
    print("All assertions passed.")
}

run()
