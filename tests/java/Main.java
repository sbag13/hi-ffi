public class Main {

    static void methodsTests() {
        try (var testStruct = new TestStruct()) {
            testStruct.publicMethod();
            testStruct.publicMethodTakingPrimitives(10, false);
            testStruct.publicMethodTakingString("Test string from caller");
            try (var methodStruct = new TestStruct2()) {
                methodStruct.setI32Field(55);
                testStruct.publicMethodTakingStruct(methodStruct);
            }
            if (testStruct.publicMethodReturningPrimitive() != 24
                    || !testStruct.publicMethodReturningBool()) {
                throw new AssertionError("TestStruct primitive method returned the wrong value");
            }
            if (!testStruct.publicMethodReturningString().equals("String returned from Rust method")) {
                throw new AssertionError("TestStruct String method returned the wrong value");
            }
            try (var methodResult = testStruct.publicMethodReturningStruct()) {
                if (methodResult.getI32Field() != 99) {
                    throw new AssertionError("TestStruct struct method returned the wrong value");
                }
            }
            if (!testStruct.comboMethod("first", "second", true).equals("first")) {
                throw new AssertionError("TestStruct combo method returned the wrong value");
            }

            testStruct.setI32Field(-5);
            if (FfiModule.takeStructAndReturnStatusBeforeTheyAreDefined(testStruct) != TestStatus.Pending) {
                throw new AssertionError("takeStructAndReturnStatusBeforeTheyAreDefined returned the wrong value");
            }
        }
    }

    static void methodResultTests() {
        try (var testStruct = new TestStruct()) {
            if (testStruct.methodWithIntResult(false) != 16) {
                throw new AssertionError("method_with_int_result returned the wrong value");
            }
            try {
                testStruct.methodWithIntResult(true);
                throw new AssertionError("method_with_int_result did not return an error");
            } catch (RustException error) {
                if (!error.toString().equals("RustException: StructError: EnumError: VariantTwo")
                        || !error.source().orElseThrow().toString()
                                .equals("RustException: EnumError: VariantTwo")) {
                    throw new AssertionError("method_with_int_result returned the wrong error chain");
                }
            }
            if (!testStruct.methodWithStringResult().equals("Ok!")
                    || testStruct.methodWithEnumResult() != TestStatus.Pending
                    || !testStruct.methodWithVecOfIntsResult().equals(
                            java.util.List.of((byte) 1, (byte) 2, (byte) 7))
                    || !testStruct.methodWithVecOfBoolsResult().equals(
                            java.util.List.of(false, true, true, true))
                    || !testStruct.methodWithVecOfStringsResult().equals(
                            java.util.List.of("some", "string"))) {
                throw new AssertionError("TestStruct result method returned the wrong value");
            }

            var structValues = TestStruct.staticMethodWithVecOfStructsResult();
            try (var first = structValues.get(0); var second = structValues.get(1)) {
                if (first.getI32Field() != 2 || second.getI32Field() != -5) {
                    throw new AssertionError("static_method_with_vec_of_structs_result returned wrong values");
                }
            }
        }

        if (!TestStruct.staticMethodWithBoolResult(false)) {
            throw new AssertionError("static_method_with_bool_result returned the wrong value");
        }
        try {
            TestStruct.staticMethodWithBoolResult(true);
            throw new AssertionError("static_method_with_bool_result did not return an error");
        } catch (RustException error) {
            if (error.source().isPresent()) {
                throw new AssertionError("SimpleError should not have a source");
            }
        }
        try (var returned = TestStruct.staticMethodWithStructResult()) {
            if (returned.getI32Field() != 267) {
                throw new AssertionError("static_method_with_struct_result returned the wrong value");
            }
        }
        if (!TestStruct.staticMethodWithVecOfBoolsResult().equals(
                    java.util.List.of(true, false, false))
                || !TestStruct.staticMethodWithVecOfEnumResult().equals(
                    java.util.List.of(TestStatus.Inactive, TestStatus.Pending, TestStatus.Active))) {
            throw new AssertionError("TestStruct static result method returned the wrong value");
        }
    }

    static void methodOptionTests() {
        try (var testStruct = new TestStruct()) {
            testStruct.methodTakingOptInt(java.util.Optional.of(20), true);
            testStruct.methodTakingOptInt(java.util.Optional.empty(), false);
            TestStruct.staticMethodTakingOptString(java.util.Optional.of("Optional string"), true);
            TestStruct.staticMethodTakingOptString(java.util.Optional.empty(), false);
            testStruct.methodTakingOptBool(java.util.Optional.of(false), true);
            testStruct.methodTakingOptBool(java.util.Optional.empty(), false);
            TestStruct.staticMethodTakingOptEnum(java.util.Optional.of(TestStatus.Active), true);
            TestStruct.staticMethodTakingOptEnum(java.util.Optional.empty(), false);
            try (var input = new TestStruct2()) {
                input.setI32Field(789);
                testStruct.methodTakingOptStruct(java.util.Optional.of(input), true);
            }
            testStruct.methodTakingOptStruct(java.util.Optional.empty(), false);

            if (!testStruct.methodReturningOptInt(true).equals(java.util.Optional.of(30))
                    || testStruct.methodReturningOptInt(false).isPresent()
                    || !TestStruct.staticMethodReturningOptBool(true)
                            .equals(java.util.Optional.of(false))
                    || TestStruct.staticMethodReturningOptBool(false).isPresent()
                    || !testStruct.methodReturningOptString(true)
                            .equals(java.util.Optional.of("Optional string from Rust"))
                    || testStruct.methodReturningOptString(false).isPresent()
                    || !TestStruct.staticMethodReturningOptEnum(true)
                            .equals(java.util.Optional.of(TestStatus.Inactive))
                    || TestStruct.staticMethodReturningOptEnum(false).isPresent()) {
                throw new AssertionError("TestStruct optional method returned the wrong value");
            }
            var someStruct = testStruct.methodReturningOptStruct(true);
            try (var returned = someStruct.orElseThrow()) {
                if (returned.getI32Field() != 654) {
                    throw new AssertionError("method_returning_opt_struct returned the wrong value");
                }
            }
            if (testStruct.methodReturningOptStruct(false).isPresent()) {
                throw new AssertionError("method_returning_opt_struct should return empty");
            }
        }
    }

    static void staticMethodsTests() {
        TestStruct.staticMethod();
        TestStruct.staticMethodTakingPrimitives(20, true);
        TestStruct.staticMethodTakingString("static method string");
        TestStruct.staticMethodTakingStruct(TestStruct2.newTs2(77));
        if (TestStruct.staticMethodReturningPrimitive() != 22
                || !TestStruct.staticMethodReturningString().equals("String returned from Rust static method")) {
            throw new AssertionError("TestStruct static method returned the wrong value");
        }
        try (var staticResult = TestStruct.staticMethodReturningStruct()) {
            if (staticResult.getI32Field() != 77) {
                throw new AssertionError("TestStruct static struct method returned the wrong value");
            }
        }

        // TestStruct4 has `impl TestStruct4` appearing BEFORE the struct definition
        // (tests the WAITING_FOR_WRAPPERS mechanism)
        try (var ts4 = TestStruct4.staticMethodReturningSelf(123)) {
            if (ts4.getI32Field() != 123) {
                throw new AssertionError("TestStruct4 static method returned the wrong value");
            }
        }
    }

    static void vectorMethodsTests() {
        // instance method with/returning vectors
        try (var testStruct = new TestStruct()) {
            testStruct.publicMethodTakingVecOfPrimitives(java.util.List.of(1, 2, 3, 4, 5));
            testStruct.publicMethodTakingVecOfStrings(java.util.List.of("Hello", "World"));
            if (!testStruct.publicMethodReturningVecOfPrimitives().equals(java.util.List.of(10, 20, 30, 40, 50))) {
                throw new AssertionError("TestStruct primitive vector method returned the wrong value");
            }
        }

        // static method with/returning vectors
        TestStruct.staticMethodTakingVecOfPrimitives(
                java.util.List.of((short) 6, (short) 7, (short) 8, (short) 9, (short) 10));
        TestStruct.staticMethodTakingVecOfStrings(java.util.List.of("Static", "Method"));
        try (var first = new TestStruct2(); var second = new TestStruct2()) {
            first.setI32Field(100);
            second.setI32Field(200);
            TestStruct.staticMethodTakingVecOfStructs(java.util.List.of(first, second));
        }
        if (!TestStruct.staticMethodReturningVecOfPrimitives().equals(java.util.List.of(60, 70, 80, 90, 100))) {
            throw new AssertionError("TestStruct static primitive vector method returned the wrong value");
        }
        if (!TestStruct.staticMethodReturningVecOfStrings().equals(java.util.List.of("Static", "Method", "Vector"))) {
            throw new AssertionError("TestStruct static String vector method returned the wrong value");
        }

        // struct with vector fields
        try (var vectors = new StructWithVecs()) {
            vectors.setVecOfInts(java.util.List.of(1, 2, 3));
            if (!vectors.getVecOfInts().equals(java.util.List.of(1, 2, 3))) {
                throw new AssertionError("StructWithVecs integer vector field failed");
            }
            vectors.setVecOfBools(java.util.List.of(true, false));
            if (!vectors.getVecOfBools().equals(java.util.List.of(true, false))) {
                throw new AssertionError("StructWithVecs boolean vector field failed");
            }
            vectors.setVecOfStrings(java.util.List.of("one", "two"));
            if (!vectors.getVecOfStrings().equals(java.util.List.of("one", "two"))) {
                throw new AssertionError("StructWithVecs String vector field failed");
            }
            vectors.setVecOfEnums(java.util.List.of(TestStatus.Active, TestStatus.Pending));
            if (!vectors.getVecOfEnums().equals(java.util.List.of(TestStatus.Active, TestStatus.Pending))) {
                throw new AssertionError("StructWithVecs enum vector field failed");
            }
            try (var first = new TestStruct2(); var second = new TestStruct2()) {
                first.setI32Field(7);
                second.setI32Field(8);
                vectors.setVecOfStructs(java.util.List.of(first, second));
            }
            var returnedStructs = vectors.getVecOfStructs();
            try (var first = returnedStructs.get(0); var second = returnedStructs.get(1)) {
                if (first.getI32Field() != 7 || second.getI32Field() != 8) {
                    throw new AssertionError("StructWithVecs struct vector field failed");
                }
            }
        }
    }

    static void structTests() {
        try (var testStruct = new TestStruct()) {
            testStruct.setI32Field(12);
            if (testStruct.getI32Field() != 12) {
                throw new AssertionError("TestStruct i32_field accessor failed");
            }
            testStruct.setBoolField(true);
            if (!testStruct.getBoolField()) {
                throw new AssertionError("TestStruct bool_field accessor failed");
            }
            testStruct.setStringField("Java field value");
            if (!testStruct.getStringField().equals("Java field value")) {
                throw new AssertionError("TestStruct string_field accessor failed");
            }
            if (testStruct.getNoSetter() != 0) {
                throw new AssertionError("TestStruct getter-only field accessor failed");
            }
            testStruct.setF32Field(3.5f);
            if (testStruct.getF32Field() != 3.5f) {
                throw new AssertionError("TestStruct f32_field accessor failed");
            }
            try (var nested = new TestStruct2()) {
                nested.setI32Field(91);
                testStruct.setStructField(nested);
                try (var returnedNested = testStruct.getStructField()) {
                    if (returnedNested.getI32Field() != 91) {
                        throw new AssertionError("TestStruct nested struct accessor failed");
                    }
                }
            }
        }
    }

    static void statusStructTests() {
        try (var status = StructWithStatus.inactivate()) {
            if (status.getStatus() != TestStatus.Inactive) {
                throw new AssertionError("StructWithStatus.inactivate returned the wrong status");
            }
            status.setStatus(TestStatus.Active);
            if (status.getStatus() != TestStatus.Active) {
                throw new AssertionError("StructWithStatus status setter/getter returned the wrong value");
            }
        }
    }

    static void partialEqTests() {
        try (var testStruct = new TestStruct()) {
            try (var equalStruct = new TestStruct()) {
                if (!testStruct.equals(equalStruct)) {
                    throw new AssertionError("Rust PartialEq equality returned false");
                }
                equalStruct.setI32Field(99);
                if (testStruct.equals(equalStruct)) {
                    throw new AssertionError("Rust PartialEq equality returned true");
                }
            }
        }
    }

    static void defaultImplTests() {
        // TestStruct4 has `impl Default` appearing BEFORE the struct definition,
        // so `new TestStruct4()` must be backed by the generated default constructor.
        try (var ts4 = new TestStruct4()) {
            if (ts4.getI32Field() != 99) {
                throw new AssertionError("TestStruct4 default constructor returned the wrong value");
            }
        }
    }

    static void functionsTests() {
        FfiModule.simpleFunction();
        FfiModule.takeStatusBeforeItIsDefined(TestStatus.Pending);

        FfiModule.functionWithPrimitiveArgs(100, true);
        FfiModule.functionWithStringArg("Hello, World!");
        FfiModule.functionWithPrimitiveAndStringArg(42, false, "Complex function!");

        try (var input = new TestStruct2()) {
            input.setI32Field(55);
            FfiModule.functionTakingStruct(input);
        }

        try (var output = FfiModule.functionReturningStruct()) {
            if (output.getI32Field() != 48) {
                throw new AssertionError("function_returning_struct returned the wrong value");
            }
        }

        if (FfiModule.functionReturnPrimitive() != 42) {
            throw new AssertionError("function_return_primitive returned the wrong value");
        }
        if (FfiModule.functionReturnFloat() != 5.21) {
            throw new AssertionError("function_return_float returned the wrong value");
        }
        if (FfiModule.functionReturnNegatedBool(true)) {
            throw new AssertionError("function_return_negated_bool returned the wrong value");
        }
        if (!FfiModule.functionReturnNegatedBool(false)) {
            throw new AssertionError("function_return_negated_bool returned the wrong value");
        }
        if (!FfiModule.functionReturnString().equals("String returned from Rust")) {
            throw new AssertionError("function_return_string returned the wrong value");
        }
        if (!FfiModule.comboFunction("first", "second", true, new TestStruct()).equals("first")) {
            throw new AssertionError("combo_function returned the wrong value");
        }
        if (!FfiModule.comboStructFunction(new TestStruct(), new TestStruct(), new TestStruct2())
                .equals(new TestStruct())) {
            throw new AssertionError("combo_struct_function returned the wrong value");
        }
    }

    static void optionTests() {
        FfiModule.functionTakingSomeInt(java.util.Optional.of(10), true);
        FfiModule.functionTakingSomeInt(java.util.Optional.empty(), false);
        FfiModule.functionTakingSomeBool(java.util.Optional.of(true), true);
        FfiModule.functionTakingSomeBool(java.util.Optional.empty(), false);
        FfiModule.functionTakingSomeString(java.util.Optional.of("Some string"), true);
        FfiModule.functionTakingSomeString(java.util.Optional.empty(), false);
        FfiModule.functionTakingSomeEnum(java.util.Optional.of(TestStatus.Pending), true);
        FfiModule.functionTakingSomeEnum(java.util.Optional.empty(), false);
        try (var input = new TestStruct()) {
            input.setI32Field(567);
            FfiModule.functionTakingSomeStruct(java.util.Optional.of(input), true);
            FfiModule.functionTakingSomeStruct(java.util.Optional.empty(), false);
        }

        if (FfiModule.functionReturningOptInt(true).orElseThrow() != 100
                || FfiModule.functionReturningOptInt(false).isPresent()) {
            throw new AssertionError("function_returning_opt_int returned the wrong value");
        }
        if (!FfiModule.functionReturningOptBool(true).orElseThrow()
                || FfiModule.functionReturningOptBool(false).isPresent()) {
            throw new AssertionError("function_returning_opt_bool returned the wrong value");
        }
        if (!FfiModule.functionReturningOptString(true).orElseThrow().equals("Some Rust String")
                || FfiModule.functionReturningOptString(false).isPresent()) {
            throw new AssertionError("function_returning_opt_string returned the wrong value");
        }
        if (FfiModule.functionReturningOptEnum(true).orElseThrow() != TestStatus.Pending
                || FfiModule.functionReturningOptEnum(false).isPresent()) {
            throw new AssertionError("function_returning_opt_enum returned the wrong value");
        }
        var returnedStruct = FfiModule.functionReturningOptStruct(true);
        try (var value = returnedStruct.orElseThrow()) {
            if (value.getI32Field() != 234) {
                throw new AssertionError("function_returning_opt_struct returned the wrong value");
            }
        }
        if (FfiModule.functionReturningOptStruct(false).isPresent()) {
            throw new AssertionError("function_returning_opt_struct returned the wrong value");
        }
    }

    static void optionFieldTests() {
        try (var options = new StructWithOptions()) {
            if (options.getOptInt().isPresent()
                    || options.getOptBool().isPresent()
                    || options.getOptString().isPresent()
                    || options.getOptEnum().isPresent()
                    || options.getOptStruct().isPresent()) {
                throw new AssertionError("StructWithOptions fields should initially be empty");
            }

            options.setOptInt(java.util.Optional.of(555));
            options.setOptBool(java.util.Optional.of(true));
            options.setOptString(java.util.Optional.of("Optional string field"));
            options.setOptEnum(java.util.Optional.of(TestStatus.Pending));
            try (var input = new TestStruct2()) {
                input.setI32Field(321);
                options.setOptStruct(java.util.Optional.of(input));
            }

            if (!options.getOptInt().equals(java.util.Optional.of(555))
                    || !options.getOptBool().equals(java.util.Optional.of(true))
                    || !options.getOptString().equals(java.util.Optional.of("Optional string field"))
                    || !options.getOptEnum().equals(java.util.Optional.of(TestStatus.Pending))) {
                throw new AssertionError("StructWithOptions primitive fields returned the wrong value");
            }

            var returnedStruct = options.getOptStruct();
            try (var value = returnedStruct.orElseThrow()) {
                if (value.getI32Field() != 321) {
                    throw new AssertionError("StructWithOptions struct field returned the wrong value");
                }
            }

            options.setOptInt(java.util.Optional.empty());
            options.setOptBool(java.util.Optional.empty());
            options.setOptString(java.util.Optional.empty());
            options.setOptEnum(java.util.Optional.empty());
            options.setOptStruct(java.util.Optional.empty());
            if (options.getOptInt().isPresent()
                    || options.getOptBool().isPresent()
                    || options.getOptString().isPresent()
                    || options.getOptEnum().isPresent()
                    || options.getOptStruct().isPresent()) {
                throw new AssertionError("StructWithOptions fields should be empty after clearing");
            }
        }
    }

    static void vectorTests() {
        FfiModule.functionTakingVecOfPrimitives(java.util.List.of(1, 2, 3, 4, 5));
        FfiModule.functionTakingVecOfBools(java.util.List.of(true, false, true, true));
        FfiModule.functionTakingVecOfEnums(java.util.List.of(
                TestStatus.Active, TestStatus.Inactive, TestStatus.Pending, TestStatus.Active));
        FfiModule.functionTakingVecOfStrings(java.util.List.of(
                "Hello, Rust!", "Hello, C++!", "Hello, Python!", "Hello, Swift!"));
        try (var first = new TestStruct(); var second = new TestStruct()) {
            first.setI32Field(15);
            second.setI32Field(17);
            FfiModule.functionTakingVecOfStructs(java.util.List.of(first, second));
        }

        if (!FfiModule.functionReturningVecOfInt().equals(java.util.List.of(3, 2, 7, 8))) {
            throw new AssertionError("function_returning_vec_of_int returned the wrong value");
        }
        if (!FfiModule.functionReturningVecOfBool().equals(java.util.List.of(true, false, true, true))) {
            throw new AssertionError("function_returning_vec_of_bool returned the wrong value");
        }
        if (!FfiModule.functionReturningVecOfEnums().equals(java.util.List.of(
                TestStatus.Pending, TestStatus.Active, TestStatus.Inactive, TestStatus.Active))) {
            throw new AssertionError("function_returning_vec_of_enums returned the wrong value");
        }
        if (!FfiModule.functionReturningVecOfString().equals(java.util.List.of("Hello", "World", "Rust"))) {
            throw new AssertionError("function_returning_vec_of_string returned the wrong value");
        }
        var returnedStructs = FfiModule.functionReturningVecOfStructs();
        try (var first = returnedStructs.get(0); var second = returnedStructs.get(1)) {
            if (first.getI32Field() != 8 || second.getI32Field() != 11) {
                throw new AssertionError("function_returning_vec_of_structs returned the wrong value");
            }
        }
    }

    static void enumTests() {
        if (FfiModule.getStatus() != TestStatus.Active) {
            throw new AssertionError("get_status returned the wrong value");
        }
        if (FfiModule.getInactiveStatus() != TestStatus.Inactive) {
            throw new AssertionError("get_inactive_status returned the wrong value");
        }
        FfiModule.assertActive(TestStatus.Active);
        FfiModule.assertInactive(TestStatus.Inactive);
    }

    static void resultTests() {
        FfiModule.functionWithUnitExpressionResult(); // just no errors
        if (FfiModule.functionWithPrimitiveResult(false) != 123) {
            throw new AssertionError("function_with_primitive_result returned the wrong value");
        }
        try {
            FfiModule.functionWithPrimitiveResult(true);
            throw new AssertionError("function_with_primitive_result did not return an error");
        } catch (RustException e) {
            if (!e.toString().equals("RustException: StructError: EnumError: VariantTwo")) {
                throw new AssertionError("function_with_primitive_result returned the wrong error");
            }
            var source = e.source().orElseThrow();
            if (!source.toString().equals("RustException: EnumError: VariantTwo")) {
                throw new AssertionError("function_with_primitive_result returned the wrong source error");
            }
            var sourceOfSource = source.source().orElseThrow();
            if (!sourceOfSource.toString().equals("RustException: SimpleError")) {
                throw new AssertionError("function_with_primitive_result returned the wrong source of source error");
            }
            var sourceOfSourceOfSource = sourceOfSource.source();
            if (sourceOfSourceOfSource.isPresent()) {
                throw new AssertionError("SimpleError should not have a source");
            }
        }

        if (!FfiModule.functionWithBoolResult(false)) {
            throw new AssertionError("function_with_bool_result returned the wrong value");
        }
        try {
            FfiModule.functionWithBoolResult(true);
            throw new AssertionError("function_with_bool_result did not return an error");
        } catch (RustException e) {
            if (!e.toString().equals("RustException: SimpleError")) {
                throw new AssertionError("function_with_bool_result returned the wrong error");
            }
            if (e.source().isPresent()) {
                throw new AssertionError("SimpleError should not have a source");
            }
        }

        if (!FfiModule.functionWithStringResult(false).equals("No error")) {
            throw new AssertionError("function_with_string_result returned the wrong value");
        }
        try {
            FfiModule.functionWithStringResult(true);
            throw new AssertionError("function_with_string_result did not return an error");
        } catch (RustException e) {
            if (!e.toString().equals("RustException: EnumError: VariantOne")) {
                throw new AssertionError("function_with_string_result returned the wrong error");
            }
        }

        if (FfiModule.functionWithStructResult(false).getI32Field() != 256) {
            throw new AssertionError("function_with_struct_result returned the wrong value");
        }
        try {
            FfiModule.functionWithStructResult(true);
            throw new AssertionError("function_with_struct_result did not return an error");
        } catch (RustException e) {
            if (!e.toString().equals("RustException: StructError: EnumError: VariantTwo")) {
                throw new AssertionError("function_with_struct_result returned the wrong error");
            }
        }

        if (FfiModule.functionWithEnumResult(false) != TestStatus.Pending) {
            throw new AssertionError("function_with_enum_result returned the wrong value");
        }
        try {
            FfiModule.functionWithEnumResult(true);
            throw new AssertionError("function_with_enum_result did not return an error");
        } catch (RustException e) {
            if (!e.toString().equals("RustException: EnumError: VariantOne")) {
                throw new AssertionError("function_with_enum_result returned the wrong error");
            }
        }

        if (!FfiModule.functionWithVecIntResult(false).equals(java.util.List.of(10, 20, 30))
                || !FfiModule.functionWithVecBoolResult(false).equals(
                        java.util.List.of(true, false, false))
                || !FfiModule.functionWithVecStringResult(false).equals(
                        java.util.List.of("One", "Two", "Three"))
                || !FfiModule.functionWithVecEnumResult(false).equals(java.util.List.of(
                        TestStatus.Pending, TestStatus.Active, TestStatus.Inactive))) {
            throw new AssertionError("vector result function returned the wrong value");
        }

        var returnedStructs = FfiModule.functionWithVecStructResult(false);
        try (var first = returnedStructs.get(0); var second = returnedStructs.get(1)) {
            if (first.getI32Field() != 512 || second.getI32Field() != 1024) {
                throw new AssertionError("function_with_vec_struct_result returned the wrong value");
            }
        }

        assertVectorResultError(() -> FfiModule.functionWithVecIntResult(true),
                "RustException: StructError: EnumError: VariantTwo");
        assertVectorResultError(() -> FfiModule.functionWithVecBoolResult(true),
                "RustException: SimpleError");
        assertVectorResultError(() -> FfiModule.functionWithVecStringResult(true),
                "RustException: EnumError: VariantOne");
        assertVectorResultError(() -> FfiModule.functionWithVecStructResult(true),
                "RustException: StructError: EnumError: VariantTwo");
        assertVectorResultError(() -> FfiModule.functionWithVecEnumResult(true),
                "RustException: EnumError: VariantOne");
    }

    @FunctionalInterface
    interface ThrowingCall {
        Object call();
    }

    static void assertVectorResultError(ThrowingCall call, String expected) {
        try {
            call.call();
            throw new AssertionError("vector result function did not return an error");
        } catch (RustException error) {
            if (!error.toString().equals(expected)) {
                throw new AssertionError("Expected " + expected + ", got " + error);
            }
        }
    }

    static final class TraitHandler implements java.lang.reflect.InvocationHandler {
        private final java.util.List<TestStruct2> createdStructs = new java.util.ArrayList<>();

        RustTrait create() {
            return (RustTrait) java.lang.reflect.Proxy.newProxyInstance(
                    RustTrait.class.getClassLoader(),
                    new Class<?>[] {RustTrait.class},
                    this);
        }

        private TestStruct2 struct(int value) {
            var result = TestStruct2.newTs2(value);
            createdStructs.add(result);
            return result;
        }

        private static void check(boolean valid, String method) {
            if (!valid) {
                throw new AssertionError("Unexpected arguments for " + method);
            }
        }

        private static void failIfRequested(boolean fail) {
            if (fail) {
                throw new IllegalStateException("Error from trait object");
            }
        }

        @Override
        public Object invoke(Object proxy, java.lang.reflect.Method method, Object[] args) {
            Object[] values = args == null ? new Object[0] : args;
            return switch (method.getName()) {
                case "traitSimpleFn" -> null;
                case "traitFnWithSimpleArgs" -> {
                    check(values[0].equals(42)
                                    && values[1].equals(4.2)
                                    && values[2] == TestStatus.Pending
                                    && values[3].equals(true),
                            method.getName());
                    yield null;
                }
                case "traitFnWithStringArg" -> {
                    check(values[0].equals("Hello from trait object"), method.getName());
                    yield null;
                }
                case "traitFnWithStructArg" -> {
                    check(((TestStruct) values[0]).getI32Field() == 123, method.getName());
                    yield null;
                }
                case "traitFnWithVecOfPrimitives" -> {
                    check(values[0].equals(java.util.List.of(1, 2, 3, 4, 5)), method.getName());
                    yield null;
                }
                case "traitFnWithVecOfBools" -> {
                    check(values[0].equals(java.util.List.of(true, false, true)), method.getName());
                    yield null;
                }
                case "traitFnWithVecOfEnums" -> {
                    check(values[0].equals(java.util.List.of(TestStatus.Active, TestStatus.Inactive)), method.getName());
                    yield null;
                }
                case "traitFnWithVecOfStrings" -> {
                    check(values[0].equals(java.util.List.of("Hello", "Trait", "Object")), method.getName());
                    yield null;
                }
                case "traitFnWithVecOfStructs" -> {
                    var structs = (java.util.List<TestStruct2>) values[0];
                    try (var first = structs.get(0); var second = structs.get(1)) {
                        check(first.getI32Field() == 321 && second.getI32Field() == 654, method.getName());
                    }
                    yield null;
                }
                case "traitFnWithOptions" -> {
                    var optionalStruct = (java.util.Optional<TestStruct2>) values[4];
                    check(values[0].equals(java.util.Optional.of(42L))
                                    && values[1].equals(java.util.Optional.of("Hello from trait object"))
                                    && values[2].equals(java.util.Optional.of(false))
                                    && values[3].equals(java.util.Optional.of(TestStatus.Pending))
                                    && optionalStruct.isPresent()
                                    && optionalStruct.orElseThrow().getI32Field() == 789,
                            method.getName());
                    optionalStruct.ifPresent(TestStruct2::close);
                    yield null;
                }
                case "traitFnReturnInt" -> 12345;
                case "traitFnReturnBool" -> true;
                case "traitFnReturnString" -> "String from trait object";
                case "traitFnReturnStruct" -> struct(987);
                case "traitFnReturnEnum" -> TestStatus.Inactive;
                case "traitFnReturnVecOfPrimitives" -> java.util.List.of(10, 20, 30);
                case "traitFnReturnVecOfBools" -> java.util.List.of(true, false, true, true);
                case "traitFnReturnVecOfEnums" -> java.util.List.of(
                        TestStatus.Active, TestStatus.Inactive, TestStatus.Pending);
                case "traitFnReturnVecOfStrings" -> java.util.List.of("Hello", "from", "trait", "object");
                case "traitFnReturnVecOfStructs" -> java.util.List.of(struct(111), struct(222));
                case "traitFnReturningOptionInt" -> (boolean) values[0]
                        ? java.util.Optional.of(555) : java.util.Optional.empty();
                case "traitFnReturningOptionBool" -> (boolean) values[0]
                        ? java.util.Optional.of(false) : java.util.Optional.empty();
                case "traitFnReturningOptionString" -> (boolean) values[0]
                        ? java.util.Optional.of("Some string") : java.util.Optional.empty();
                case "traitFnReturningOptionEnum" -> (boolean) values[0]
                        ? java.util.Optional.of(TestStatus.Pending) : java.util.Optional.empty();
                case "traitFnReturningOptionStruct" -> (boolean) values[0]
                        ? java.util.Optional.of(struct(789)) : java.util.Optional.empty();
                case "traitFnReturningResultInt" -> {
                    failIfRequested((boolean) values[0]);
                    yield 555;
                }
                case "traitFnReturningResultBool" -> {
                    failIfRequested((boolean) values[0]);
                    yield true;
                }
                case "traitFnReturningResultString" -> {
                    failIfRequested((boolean) values[0]);
                    yield "Some string";
                }
                case "traitFnReturningResultStruct" -> {
                    failIfRequested((boolean) values[0]);
                    yield struct(789);
                }
                case "traitFnReturningResultEnum" -> {
                    failIfRequested((boolean) values[0]);
                    yield TestStatus.Active;
                }
                case "traitFnReturningResultVecOfInts" -> {
                    failIfRequested((boolean) values[0]);
                    yield java.util.List.of(1, 2, 3);
                }
                case "traitFnReturningResultVecOfBools" -> {
                    failIfRequested((boolean) values[0]);
                    yield java.util.List.of(true, false, true);
                }
                case "traitFnReturningResultVecOfEnums" -> {
                    failIfRequested((boolean) values[0]);
                    yield java.util.List.of(TestStatus.Active, TestStatus.Inactive, TestStatus.Pending);
                }
                case "traitFnReturningResultVecOfStrings" -> {
                    failIfRequested((boolean) values[0]);
                    yield java.util.List.of("Hello", "from", "trait", "object");
                }
                case "traitFnReturningResultVecOfStructs" -> {
                    failIfRequested((boolean) values[0]);
                    yield java.util.List.of(struct(111), struct(222));
                }
                case "traitFnReturningResultWithUnitExpression" -> {
                    failIfRequested((boolean) values[0]);
                    yield null;
                }
                case "close" -> null;
                default -> throw new UnsupportedOperationException(method.getName());
            };
        }

        void closeCreatedStructs() {
            createdStructs.forEach(TestStruct2::close);
        }
    }

    static void traitTests() {
        var handler = new TraitHandler();
        var implementation = handler.create();
        FfiModule.functionTakingTraitObject(implementation);
        if (implementation.traitFnReturnInt() != 12345) {
            throw new AssertionError("Java trait implementation was not usable after the Rust call");
        }
        handler.closeCreatedStructs();
        try (var receiver = new TestStruct()) {
            receiver.methodTakingTraitObject(implementation);
            TestStruct.staticMethodTakingTraitObject(implementation);
            try (var returned = receiver.methodReturningTraitObject()) {
                if (returned.traitFnReturnInt() != 12345) {
                    throw new AssertionError("TestStruct returned the wrong Rust trait object");
                }
            }
            try (var returned = TestStruct.staticMethodReturningTraitObject()) {
                if (returned.traitFnReturnInt() != 12345) {
                    throw new AssertionError("TestStruct static method returned the wrong trait object");
                }
            }
        }
        handler.closeCreatedStructs();

        try (var rustTrait = FfiModule.functionReturningTraitObject()) {
            rustTrait.traitSimpleFn();
            rustTrait.traitFnWithSimpleArgs(42, 4.2, TestStatus.Pending, true);
            rustTrait.traitFnWithStringArg("Hello from trait object");
            try (var input = new TestStruct()) {
                input.setI32Field(123);
                rustTrait.traitFnWithStructArg(input);
            }
            rustTrait.traitFnWithVecOfPrimitives(java.util.List.of(1, 2, 3, 4, 5));
            rustTrait.traitFnWithVecOfBools(java.util.List.of(true, false, true));
            rustTrait.traitFnWithVecOfEnums(java.util.List.of(TestStatus.Active, TestStatus.Inactive));
            rustTrait.traitFnWithVecOfStrings(java.util.List.of("Hello", "Trait", "Object"));
            try (var first = TestStruct2.newTs2(321); var second = TestStruct2.newTs2(654)) {
                rustTrait.traitFnWithVecOfStructs(java.util.List.of(first, second));
            }
            try (var input = TestStruct2.newTs2(789)) {
                rustTrait.traitFnWithOptions(java.util.Optional.of(42L),
                        java.util.Optional.of("Hello from trait object"),
                        java.util.Optional.of(false), java.util.Optional.of(TestStatus.Pending),
                        java.util.Optional.of(input));
            }
            if (rustTrait.traitFnReturnInt() != 12345
                    || !rustTrait.traitFnReturnBool()
                    || !rustTrait.traitFnReturnString().equals("String from trait object")
                    || rustTrait.traitFnReturnEnum() != TestStatus.Inactive
                    || !rustTrait.traitFnReturnVecOfPrimitives().equals(java.util.List.of(10, 20, 30))
                    || !rustTrait.traitFnReturnVecOfBools().equals(java.util.List.of(true, false, true, true))
                    || !rustTrait.traitFnReturnVecOfEnums().equals(java.util.List.of(
                            TestStatus.Active, TestStatus.Inactive, TestStatus.Pending))
                    || !rustTrait.traitFnReturnVecOfStrings().equals(
                            java.util.List.of("Hello", "from", "trait", "object"))) {
                throw new AssertionError("Rust trait proxy returned the wrong value");
            }
            try (var returned = rustTrait.traitFnReturnStruct()) {
                if (returned.getI32Field() != 987) {
                    throw new AssertionError("Rust trait proxy returned the wrong struct");
                }
            }
            var returnedStructs = rustTrait.traitFnReturnVecOfStructs();
            try (var first = returnedStructs.get(0); var second = returnedStructs.get(1)) {
                if (first.getI32Field() != 111 || second.getI32Field() != 222) {
                    throw new AssertionError("Rust trait proxy returned the wrong struct vector");
                }
            }
            if (!rustTrait.traitFnReturningOptionInt(true).equals(java.util.Optional.of(555))
                    || rustTrait.traitFnReturningOptionInt(false).isPresent()
                    || !rustTrait.traitFnReturningOptionBool(true).equals(java.util.Optional.of(false))
                    || rustTrait.traitFnReturningOptionBool(false).isPresent()
                    || !rustTrait.traitFnReturningOptionString(true).equals(java.util.Optional.of("Some string"))
                    || rustTrait.traitFnReturningOptionString(false).isPresent()
                    || !rustTrait.traitFnReturningOptionEnum(true).equals(java.util.Optional.of(TestStatus.Pending))
                    || rustTrait.traitFnReturningOptionEnum(false).isPresent()) {
                throw new AssertionError("Rust trait proxy returned the wrong option");
            }
            try (var returned = rustTrait.traitFnReturningOptionStruct(true).orElseThrow()) {
                if (returned.getI32Field() != 789) {
                    throw new AssertionError("Rust trait proxy returned the wrong optional struct");
                }
            }
            if (rustTrait.traitFnReturningOptionStruct(false).isPresent()
                    || rustTrait.traitFnReturningResultInt(false) != 555
                    || !rustTrait.traitFnReturningResultBool(false)
                    || !rustTrait.traitFnReturningResultString(false).equals("Some string")
                    || rustTrait.traitFnReturningResultEnum(false) != TestStatus.Active
                    || !rustTrait.traitFnReturningResultVecOfInts(false).equals(java.util.List.of(1, 2, 3))
                    || !rustTrait.traitFnReturningResultVecOfBools(false).equals(
                            java.util.List.of(true, false, true))
                    || !rustTrait.traitFnReturningResultVecOfEnums(false).equals(
                            java.util.List.of(TestStatus.Active, TestStatus.Inactive, TestStatus.Pending))
                    || !rustTrait.traitFnReturningResultVecOfStrings(false).equals(
                            java.util.List.of("Hello", "from", "trait", "object"))) {
                throw new AssertionError("Rust trait proxy returned the wrong result");
            }
            try (var returned = rustTrait.traitFnReturningResultStruct(false)) {
                if (returned.getI32Field() != 789) {
                    throw new AssertionError("Rust trait proxy returned the wrong result struct");
                }
            }
            var returnedResultStructs = rustTrait.traitFnReturningResultVecOfStructs(false);
            try (var first = returnedResultStructs.get(0); var second = returnedResultStructs.get(1)) {
                if (first.getI32Field() != 111 || second.getI32Field() != 222) {
                    throw new AssertionError("Rust trait proxy returned the wrong result struct vector");
                }
            }
            rustTrait.traitFnReturningResultWithUnitExpression(false);
            assertVectorResultError(() -> rustTrait.traitFnReturningResultInt(true),
                    "RustException: StructError: EnumError: VariantTwo");
            assertVectorResultError(() -> rustTrait.traitFnReturningResultBool(true),
                    "RustException: SimpleError");
            assertVectorResultError(() -> rustTrait.traitFnReturningResultString(true),
                    "RustException: SimpleError");
            assertVectorResultError(() -> rustTrait.traitFnReturningResultStruct(true),
                    "RustException: Error from trait object");
            assertVectorResultError(() -> rustTrait.traitFnReturningResultEnum(true),
                    "RustException: Error from trait object");
            assertVectorResultError(() -> rustTrait.traitFnReturningResultVecOfInts(true),
                    "RustException: Error from trait object");
            assertVectorResultError(() -> rustTrait.traitFnReturningResultVecOfBools(true),
                    "RustException: Error from trait object");
            assertVectorResultError(() -> rustTrait.traitFnReturningResultVecOfEnums(true),
                    "RustException: Error from trait object");
            assertVectorResultError(() -> rustTrait.traitFnReturningResultVecOfStrings(true),
                    "RustException: Error from trait object");
            assertVectorResultError(() -> rustTrait.traitFnReturningResultVecOfStructs(true),
                    "RustException: Error from trait object");
            assertVectorResultError(() -> {
                rustTrait.traitFnReturningResultWithUnitExpression(true);
                return null;
            },
                    "RustException: SimpleError");
        }
    }

    public static void main(String[] args) {
        structTests();
        statusStructTests();
        defaultImplTests();
        partialEqTests();
        functionsTests();
        methodResultTests();
        methodOptionTests();
        optionTests();
        optionFieldTests();
        methodsTests();
        staticMethodsTests();
        vectorMethodsTests();
        vectorTests();
        enumTests();
        resultTests();
        traitTests();
        System.out.println("All tests passed!");
    }
}