using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Collections;

internal static class Program
{
    private static void Main()
    {
        FunctionAndPrimitiveTests();
        VectorAndNestedContainerTests();
        HashMapTests();
        FunctionAndStructTests();
        ResultTests();
        OptionAndEnumTests();
        NestedFieldAndMethodTests();
        TraitTests();
        Console.WriteLine("C# integration tests passed");
    }
    public class RustTraitTestProxy : DispatchProxy
    {
        protected override object? Invoke(MethodInfo? targetMethod, object?[]? args)
        {
            if (targetMethod is null) throw new InvalidOperationException("Missing trait method.");
            var values = args ?? Array.Empty<object?>();
            try
            {
                switch (targetMethod.Name)
                {
                    case "TraitSimpleFn":
                        return null;
                    case "TraitFnWithSimpleArgs":
                        Check((int)values[0]! == 42 && (double)values[1]! == 4.2
                            && (TestStatus)values[2]! == TestStatus.Pending && (bool)values[3]!,
                            targetMethod.Name);
                        return null;
                    case "TraitFnWithStringArg":
                        Check((string)values[0]! == "Hello from trait object", targetMethod.Name);
                        return null;
                    case "TraitFnWithStructArg":
                        Check(((TestStruct)values[0]!).I32Field == 123, targetMethod.Name);
                        return null;
                    case "TraitFnWithVecOfPrimitives":
                        Check(((IReadOnlyList<int>)values[0]!).SequenceEqual(new[] { 1, 2, 3, 4, 5 }),
                            targetMethod.Name);
                        return null;
                    case "TraitFnWithVecOfBools":
                        Check(((IReadOnlyList<bool>)values[0]!).SequenceEqual(new[] { true, false, true }),
                            targetMethod.Name);
                        return null;
                    case "TraitFnWithVecOfEnums":
                        Check(((IReadOnlyList<TestStatus>)values[0]!)
                            .SequenceEqual(new[] { TestStatus.Active, TestStatus.Inactive }), targetMethod.Name);
                        return null;
                    case "TraitFnWithVecOfStrings":
                        Check(((IReadOnlyList<string>)values[0]!)
                            .SequenceEqual(new[] { "Hello", "Trait", "Object" }), targetMethod.Name);
                        return null;
                    case "TraitFnWithVecOfStructs":
                        var inputStructs = (IReadOnlyList<TestStruct2>)values[0]!;
                        Check(inputStructs.Select(item => item.I32Field).SequenceEqual(new[] { 321, 654 }),
                            targetMethod.Name);
                        return null;
                    case "TraitFnWithIntHashmap":
                        Check(((Dictionary<int, int>)values[0]!).Count == 2
                            && ((Dictionary<int, int>)values[0]!)[1] == 10
                            && ((Dictionary<int, int>)values[0]!)[2] == 20, targetMethod.Name);
                        return null;
                    case "TraitFnWithStringHashmap":
                        var stringMap = (Dictionary<string, string>)values[0]!;
                        Check(stringMap.Count == 2 && stringMap["alpha"] == "one" && stringMap["beta"] == "two",
                            targetMethod.Name);
                        return null;
                    case "TraitFnWithEnumHashmap":
                        var enumMap = (Dictionary<TestStatus, TestStatus>)values[0]!;
                        Check(enumMap.Count == 2 && enumMap[TestStatus.Active] == TestStatus.Pending
                            && enumMap[TestStatus.Inactive] == TestStatus.Active, targetMethod.Name);
                        return null;
                    case "TraitFnWithStructHashmap":
                        var structMap = (Dictionary<string, TestStruct2>)values[0]!;
                        Check(structMap["alpha"].I32Field == 10 && structMap["beta"].I32Field == 20,
                            targetMethod.Name);
                        return null;
                    case "TraitFnWithRecursiveMaps":
                        var mapVec = (Dictionary<string, IReadOnlyList<int>>)values[0]!;
                        var mapOption = (Dictionary<string, int?>)values[1]!;
                        var mapMap = (Dictionary<string, Dictionary<string, int>>)values[2]!;
                        var vecMap = (IReadOnlyList<Dictionary<string, int>>)values[3]!;
                        var optionMap = (Dictionary<string, int>?)values[4];
                        Check(mapVec["values"].SequenceEqual(new[] { 1, 2 })
                            && mapOption["some"] == 3 && mapOption["none"] is null
                            && mapMap["inner"]["value"] == 4 && vecMap[0]["value"] == 5
                            && optionMap?["value"] == 6, targetMethod.Name);
                        return null;
                    case "TraitFnWithOptions":
                        var optionalStruct = (TestStruct2?)values[4];
                        Check((long?)values[0] == 42 && (string?)values[1] == "Hello from trait object"
                            && (bool?)values[2] == false && (TestStatus?)values[3] == TestStatus.Pending
                            && optionalStruct?.I32Field == 789, targetMethod.Name);
                        return null;
                    case "TraitFnReturnInt": return 12345;
                    case "TraitFnReturnBool": return true;
                    case "TraitFnReturnString": return "String from trait object";
                    case "TraitFnReturnStruct": return Struct(987);
                    case "TraitFnReturnEnum": return TestStatus.Inactive;
                    case "TraitFnReturnVecOfPrimitives": return new List<int> { 10, 20, 30 };
                    case "TraitFnReturnVecOfBools": return new List<bool> { true, false, true, true };
                    case "TraitFnReturnVecOfEnums":
                        return new List<TestStatus> { TestStatus.Active, TestStatus.Inactive, TestStatus.Pending };
                    case "TraitFnReturnVecOfStrings":
                        return new List<string> { "Hello", "from", "trait", "object" };
                    case "TraitFnReturnVecOfStructs": return new List<TestStruct2> { Struct(111), Struct(222) };
                    case "TraitFnReturnIntHashmap": return new Dictionary<int, int> { [3] = 30, [4] = 40 };
                    case "TraitFnReturnStringHashmap":
                        return new Dictionary<string, string> { ["gamma"] = "three", ["delta"] = "four" };
                    case "TraitFnReturnEnumHashmap":
                        return new Dictionary<TestStatus, TestStatus>
                        {
                            [TestStatus.Active] = TestStatus.Inactive,
                            [TestStatus.Pending] = TestStatus.Active,
                        };
                    case "TraitFnReturnStructHashmap":
                        return new Dictionary<string, TestStruct2> { ["alpha"] = Struct(30), ["beta"] = Struct(40) };
                    case "TraitFnReturnMapOfVec":
                        return new Dictionary<string, IReadOnlyList<int>> { ["values"] = new[] { 16, 17 } };
                    case "TraitFnReturningOptionInt":
                        return (bool)values[0]! ? 555 : null;
                    case "TraitFnReturningOptionBool":
                        return (bool)values[0]! ? false : null;
                    case "TraitFnReturningOptionString":
                        return (bool)values[0]! ? "Some string" : null;
                    case "TraitFnReturningOptionEnum":
                        return (bool)values[0]! ? TestStatus.Pending : null;
                    case "TraitFnReturningOptionStruct":
                        return (bool)values[0]! ? Struct(789) : null;
                    case "TraitFnReturningResultInt":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return 555;
                    case "TraitFnReturningResultBool":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return true;
                    case "TraitFnReturningResultString":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return "Some string";
                    case "TraitFnReturningResultStruct":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return Struct(789);
                    case "TraitFnReturningResultEnum":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return TestStatus.Active;
                    case "TraitFnReturningResultVecOfInts":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return new List<int> { 1, 2, 3 };
                    case "TraitFnReturningResultVecOfBools":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return new List<bool> { true, false, true };
                    case "TraitFnReturningResultVecOfEnums":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return new List<TestStatus> { TestStatus.Active, TestStatus.Inactive, TestStatus.Pending };
                    case "TraitFnReturningResultVecOfStrings":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return new List<string> { "Hello", "from", "trait", "object" };
                    case "TraitFnReturningResultVecOfStructs":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return new List<TestStruct2> { Struct(111), Struct(222) };
                    case "TraitFnReturningResultWithOptionInt":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return 555;
                    case "TraitFnReturningResultWithOptionBool":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return false;
                    case "TraitFnReturningResultWithOptionString":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return "Some string";
                    case "TraitFnReturningResultWithOptionEnum":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return TestStatus.Pending;
                    case "TraitFnReturningResultWithOptionStruct":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return Struct(789);
                    case "TraitFnReturningResultWithUnitExpression":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return null;
                    case "TraitFnReturningResultWithHashmap":
                        if ((bool)values[0]!) throw new InvalidOperationException("Error from trait object");
                        return new Dictionary<string, TestStruct2> { ["alpha"] = Struct(30), ["beta"] = Struct(40) };
                    default:
                        throw new NotSupportedException($"Unsupported RustTrait member: {targetMethod.Name}");
                }
            }


            finally
            {
                foreach (var value in values) DisposeValue(value);
            }
        }

        private static TestStruct2 Struct(int value)
        {
            var result = new TestStruct2 { I32Field = value };
            return result;
        }

        private static void Check(bool condition, string method)
        {
            if (!condition) throw new InvalidOperationException($"Unexpected arguments for {method}.");
        }

        private static void DisposeValue(object? value)
        {
            if (value is IDisposable disposable)
            {
                disposable.Dispose();
            }
            else if (value is IDictionary dictionary)
            {
                foreach (DictionaryEntry entry in dictionary) DisposeValue(entry.Value);
            }
            else if (value is IEnumerable enumerable && value is not string)
            {
                foreach (var item in enumerable) DisposeValue(item);
            }
        }
    }

    private sealed class RustTraitTestImplementation : RustTrait
    {
        public void TraitSimpleFn() { }

        public void TraitFnWithSimpleArgs(int i, double f, TestStatus e, bool b) =>
        Assert(i == 42 && f == 4.2 && e == TestStatus.Pending && b, nameof(TraitFnWithSimpleArgs));

        public void TraitFnWithStringArg(string s) =>
        Assert(s == "Hello from trait object", nameof(TraitFnWithStringArg));

        public void TraitFnWithStructArg(TestStruct s)
        {
            try
            {
                Assert(s.I32Field == 123, nameof(TraitFnWithStructArg));
            }
            finally
            {
                s.Dispose();
            }
        }

        public void TraitFnWithVecOfPrimitives(IReadOnlyList<int> vec) =>
        Assert(vec.SequenceEqual(new[] { 1, 2, 3, 4, 5 }), nameof(TraitFnWithVecOfPrimitives));

        public void TraitFnWithVecOfBools(IReadOnlyList<bool> vec) =>
        Assert(vec.SequenceEqual(new[] { true, false, true }), nameof(TraitFnWithVecOfBools));

        public void TraitFnWithVecOfEnums(IReadOnlyList<TestStatus> vec) =>
        Assert(vec.SequenceEqual(new[] { TestStatus.Active, TestStatus.Inactive }), nameof(TraitFnWithVecOfEnums));

        public void TraitFnWithVecOfStrings(IReadOnlyList<string> vec) =>
        Assert(vec.SequenceEqual(new[] { "Hello", "Trait", "Object" }), nameof(TraitFnWithVecOfStrings));

        public void TraitFnWithVecOfStructs(IReadOnlyList<TestStruct2> vec)
        {
            try
            {
                Assert(vec.Select(value => value.I32Field).SequenceEqual(new[] { 321, 654 }), nameof(TraitFnWithVecOfStructs));
            }
            finally
            {
                foreach (var value in vec) value.Dispose();
            }
        }

        public void TraitFnWithIntHashmap(Dictionary<int, int> map) =>
        Assert(map.Count == 2 && map[1] == 10 && map[2] == 20, nameof(TraitFnWithIntHashmap));

        public void TraitFnWithStringHashmap(Dictionary<string, string> map) =>
        Assert(map.Count == 2 && map["alpha"] == "one" && map["beta"] == "two", nameof(TraitFnWithStringHashmap));

        public void TraitFnWithEnumHashmap(Dictionary<TestStatus, TestStatus> map) =>
        Assert(map.Count == 2 && map[TestStatus.Active] == TestStatus.Pending
            && map[TestStatus.Inactive] == TestStatus.Active, nameof(TraitFnWithEnumHashmap));

        public void TraitFnWithStructHashmap(Dictionary<string, TestStruct2> map)
        {
            try
            {
                Assert(map["alpha"].I32Field == 10 && map["beta"].I32Field == 20, nameof(TraitFnWithStructHashmap));
            }
            finally
            {
                foreach (var value in map.Values) value.Dispose();
            }
        }

        public void TraitFnWithRecursiveMaps(
        Dictionary<string, IReadOnlyList<int>> map_vec,
        Dictionary<string, int?> map_option,
        Dictionary<string, Dictionary<string, int>> map_map,
        IReadOnlyList<Dictionary<string, int>> vec_map,
        Dictionary<string, int>? option_map) =>
        Assert(map_vec["values"].SequenceEqual(new[] { 1, 2 })
            && map_option["some"] == 3 && map_option["none"] is null
            && map_map["inner"]["value"] == 4 && vec_map[0]["value"] == 5
            && option_map?["value"] == 6, nameof(TraitFnWithRecursiveMaps));

        public void TraitFnWithOptions(
        long? opt_int,
        string? opt_string,
        bool? opt_bool,
        TestStatus? opt_enum,
        TestStruct2? opt_struct)
        {
            try
            {
                Assert(opt_int == 42 && opt_string == "Hello from trait object" && opt_bool == false
                    && opt_enum == TestStatus.Pending && opt_struct?.I32Field == 789, nameof(TraitFnWithOptions));
            }
            finally
            {
                opt_struct?.Dispose();
            }
        }

        public int TraitFnReturnInt() => 12345;
        public bool TraitFnReturnBool() => true;
        public string TraitFnReturnString() => "String from trait object";
        public TestStruct2 TraitFnReturnStruct() => Struct(987);
        public TestStatus TraitFnReturnEnum() => TestStatus.Inactive;
        public List<int> TraitFnReturnVecOfPrimitives() => new() { 10, 20, 30 };
        public List<bool> TraitFnReturnVecOfBools() => new() { true, false, true, true };
        public List<TestStatus> TraitFnReturnVecOfEnums() =>
        new() { TestStatus.Active, TestStatus.Inactive, TestStatus.Pending };
        public List<string> TraitFnReturnVecOfStrings() => new() { "Hello", "from", "trait", "object" };
        public List<TestStruct2> TraitFnReturnVecOfStructs() => new() { Struct(111), Struct(222) };
        public Dictionary<int, int> TraitFnReturnIntHashmap() => new() { [3] = 30, [4] = 40 };
        public Dictionary<string, string> TraitFnReturnStringHashmap() =>
        new() { ["gamma"] = "three", ["delta"] = "four" };
        public Dictionary<TestStatus, TestStatus> TraitFnReturnEnumHashmap() =>
        new() { [TestStatus.Active] = TestStatus.Inactive, [TestStatus.Pending] = TestStatus.Active };
        public Dictionary<string, TestStruct2> TraitFnReturnStructHashmap() =>
        new() { ["alpha"] = Struct(30), ["beta"] = Struct(40) };
        public Dictionary<string, IReadOnlyList<int>> TraitFnReturnMapOfVec() =>
        new() { ["values"] = new[] { 16, 17 } };

        public int? TraitFnReturningOptionInt(bool some) => some ? 555 : null;
        public bool? TraitFnReturningOptionBool(bool some) => some ? false : null;
        public string? TraitFnReturningOptionString(bool some) => some ? "Some string" : null;
        public TestStatus? TraitFnReturningOptionEnum(bool some) => some ? TestStatus.Pending : null;
        public TestStruct2? TraitFnReturningOptionStruct(bool some) => some ? Struct(789) : null;

        public int TraitFnReturningResultInt(bool error) => Result(error, 555);
        public bool TraitFnReturningResultBool(bool error) => Result(error, true);
        public string TraitFnReturningResultString(bool error) => Result(error, "Some string");
        public TestStruct2 TraitFnReturningResultStruct(bool error) => Result(error, Struct(789));
        public TestStatus TraitFnReturningResultEnum(bool error) => Result(error, TestStatus.Active);
        public List<int> TraitFnReturningResultVecOfInts(bool error) => Result(error, new List<int> { 1, 2, 3 });
        public List<bool> TraitFnReturningResultVecOfBools(bool error) => Result(error, new List<bool> { true, false, true });
        public List<TestStatus> TraitFnReturningResultVecOfEnums(bool error) =>
        Result(error, new List<TestStatus> { TestStatus.Active, TestStatus.Inactive, TestStatus.Pending });
        public List<string> TraitFnReturningResultVecOfStrings(bool error) =>
        Result(error, new List<string> { "Hello", "from", "trait", "object" });
        public List<TestStruct2> TraitFnReturningResultVecOfStructs(bool error) =>
        Result(error, new List<TestStruct2> { Struct(111), Struct(222) });
        public int? TraitFnReturningResultWithOptionInt(bool error) => Result<int?>(error, 555);
        public bool? TraitFnReturningResultWithOptionBool(bool error) => Result<bool?>(error, false);
        public string? TraitFnReturningResultWithOptionString(bool error) => Result<string?>(error, "Some string");
        public TestStatus? TraitFnReturningResultWithOptionEnum(bool error) =>
        Result<TestStatus?>(error, TestStatus.Pending);
        public TestStruct2? TraitFnReturningResultWithOptionStruct(bool error) => Result<TestStruct2?>(error, Struct(789));

        public void TraitFnReturningResultWithUnitExpression(bool error)
        {
            if (error) throw TraitError();
        }

        public Dictionary<string, TestStruct2> TraitFnReturningResultWithHashmap(bool error) =>
        Result(error, new Dictionary<string, TestStruct2> { ["alpha"] = Struct(30), ["beta"] = Struct(40) });

        private static T Result<T>(bool error, T value)
        {
            if (error) throw TraitError();
            return value;
        }

        private static InvalidOperationException TraitError() =>
        new("Error from trait object");

        private static TestStruct2 Struct(int value) => new() { I32Field = value };
    }

    private static void FunctionAndPrimitiveTests()
    {
        FfiModule.SimpleFunction();
        FfiModule.FunctionWithPrimitiveArgs(100, true);
        FfiModule.FunctionWithPrimitiveAndStringArg(42, false, "Complex function!");
        FfiModule.FunctionWithIntegerArgs(
            -8, 8, -16, 16, -32, 32, -64, 64, (nuint)128);
        FfiModule.TakeStatusBeforeItIsDefined(TestStatus.Pending);
        using (var beforeDefinition = new TestStruct())
        {
            beforeDefinition.I32Field = -5;
            Assert(
                FfiModule.TakeStructAndReturnStatusBeforeTheyAreDefined(beforeDefinition)
                    == TestStatus.Pending,
                "forward-referenced struct and enum");
        }
    }

    private static void VectorAndNestedContainerTests()
    {
        FfiModule.FunctionTakingVecOfPrimitives(new List<int> { 1, 2, 3, 4, 5 });
        FfiModule.FunctionTakingVecOfOptions(new List<int?> { 1, null, 3 });
        Assert(
            FfiModule.FunctionReturningVecOfOptions().SequenceEqual(new int?[] { 2, null, 4 }),
            "vector of options argument and return");
        FfiModule.FunctionTakingOptionOfVec(new List<int?> { 5, null, 6 });
        Assert(
            FfiModule.FunctionReturningOptionOfVec()!.SequenceEqual(new int?[] { 8, null, 10 }),
            "option of vector argument and return");
        Assert(
            FfiModule.FunctionReturningResultOfNestedContainers()!
                .SequenceEqual(new int?[] { 12, null, 14 }),
            "Result<Option<Vec<Option<int>>>> return");
        FfiModule.FunctionTakingDeepNestedContainers(
            new List<IReadOnlyList<int?>?> { new List<int?> { 15, null }, null });
        var deepNested = FfiModule.FunctionReturningDeepNestedContainers();
        Assert(
            deepNested[0]!.SequenceEqual(new int?[] { 16, null }) && deepNested[1] is null,
            "Vec<Option<Vec<Option<int>>>> argument and return");
        FfiModule.FunctionTakingVecOfBools(new List<bool> { true, false, true, true });
    }

    private static void HashMapTests()
    {
        FfiModule.FunctionTakingHashmap(
            new Dictionary<string, int> { ["alpha"] = 10, ["beta"] = 20 });
        var returnedMap = FfiModule.FunctionReturningHashmap();
        Assert(
            returnedMap.Count == 2 && returnedMap["alpha"] == 10 && returnedMap["beta"] == 20,
            "HashMap<String, i32> return");
        Assert(FfiModule.FunctionReturningEmptyHashmap().Count == 0, "empty HashMap return");
        var mapOfVectors = FfiModule.FunctionReturningMapOfVec();
        Assert(
            mapOfVectors["values"].SequenceEqual(new[] { 10, 11 }),
            "HashMap<String, Vec<i32>> return");
        var mapOfOptions = FfiModule.FunctionReturningMapOfOption();
        Assert(
            mapOfOptions["some"] == 12 && mapOfOptions["none"] is null,
            "HashMap<String, Option<i32>> return");
        var mapOfMaps = FfiModule.FunctionReturningMapOfHashmap();
        Assert(
            mapOfMaps["inner"]["value"] == 13,
            "nested HashMap return");
        Assert(
            FfiModule.FunctionReturningOptionOfHashmap()!["value"] == 15,
            "Option<HashMap<String, i32>> return");
        Assert(
            FfiModule.FunctionReturningVecOfHashmaps()[0]["value"] == 14,
            "Vec<HashMap<String, i32>> return");
        FfiModule.FunctionTakingRecursiveMaps(
            new Dictionary<string, IReadOnlyList<int>>
            {
                ["values"] = new List<int> { 1, 2 },
            },
            new Dictionary<string, int?>
            {
                ["some"] = 3,
                ["none"] = null,
            },
            new Dictionary<string, Dictionary<string, int>>
            {
                ["inner"] = new Dictionary<string, int> { ["value"] = 4 },
            },
            new List<Dictionary<string, int>>
            {
                new() { ["value"] = 5 },
            },
            new Dictionary<string, int> { ["value"] = 6 });
        FfiModule.FunctionTakingEnumHashmap(
            new Dictionary<TestStatus, TestStatus>
            {
                [TestStatus.Active] = TestStatus.Pending,
                [TestStatus.Inactive] = TestStatus.Active,
            });
        var returnedEnumMap = FfiModule.FunctionReturningEnumHashmap();
        Assert(
            returnedEnumMap[TestStatus.Active] == TestStatus.Inactive
                && returnedEnumMap[TestStatus.Pending] == TestStatus.Active,
            "HashMap<enum, enum> return");
        using (var mapValueAlpha = new TestStruct2())
        using (var mapValueBeta = new TestStruct2())
        {
            mapValueAlpha.I32Field = 10;
            mapValueBeta.I32Field = 20;
            FfiModule.FunctionTakingStructHashmap(
                new Dictionary<string, TestStruct2>
                {
                    ["alpha"] = mapValueAlpha,
                    ["beta"] = mapValueBeta,
                });
        }
        var returnedStructMap = FfiModule.FunctionReturningStructHashmap();
        try
        {
            Assert(
                returnedStructMap["alpha"].I32Field == 10
                    && returnedStructMap["beta"].I32Field == 20,
                "HashMap<String, struct> return");
        }
        finally
        {
            foreach (var value in returnedStructMap.Values) value.Dispose();
        }
        using (var hashMapStruct = new StructWithHashMaps())
        {
            hashMapStruct.StringToInt = new Dictionary<string, int> { ["alpha"] = 30 };
            hashMapStruct.IntToString = new Dictionary<int, string> { [40] = "forty" };
            Assert(hashMapStruct.StringToInt["alpha"] == 30, "string-to-int HashMap field");
            Assert(hashMapStruct.IntToString[40] == "forty", "int-to-string HashMap field");
            using (var fieldValue = new TestStruct2())
            {
                fieldValue.I32Field = 50;
                hashMapStruct.StringToStruct =
                    new Dictionary<string, TestStruct2> { ["fifty"] = fieldValue };
            }
            var fieldStructMap = hashMapStruct.StringToStruct;
            try
            {
                Assert(fieldStructMap["fifty"].I32Field == 50, "string-to-struct HashMap field");
            }
            finally
            {
                foreach (var value in fieldStructMap.Values) value.Dispose();
            }
        }
    }

    private static void FunctionAndStructTests()
    {
        FfiModule.FunctionTakingVecOfEnums(
            new List<TestStatus> {
                TestStatus.Active, TestStatus.Inactive, TestStatus.Pending, TestStatus.Active
            });
        FfiModule.FunctionTakingVecOfStrings(
            new List<string> { "Hello, Rust!", "Hello, C++!", "Hello, Python!", "Hello, Swift!" });
        using (var first = new TestStruct())
        using (var second = new TestStruct())
        {
            first.I32Field = 15;
            second.I32Field = 17;
            FfiModule.FunctionTakingVecOfStructs(new List<TestStruct> { first, second });
        }
        Assert(
            FfiModule.FunctionReturningVecOfInt().SequenceEqual(new[] { 3, 2, 7, 8 }),
            "integer vector return");
        Assert(
            FfiModule.FunctionReturningVecOfBool().SequenceEqual(new[] { true, false, true, true }),
            "bool vector return");
        Assert(
            FfiModule.FunctionReturningVecOfEnums().SequenceEqual(
                new[] { TestStatus.Pending, TestStatus.Active, TestStatus.Inactive, TestStatus.Active }),
            "enum vector return");
        Assert(
            FfiModule.FunctionReturningVecOfString().SequenceEqual(new[] { "Hello", "World", "Rust" }),
            "string vector return");
        var returnedStructs = FfiModule.FunctionReturningVecOfStructs();
        try
        {
            Assert(
                returnedStructs.Select(value => value.I32Field).SequenceEqual(new[] { 8, 11 }),
                "struct vector return");
        }
        finally
        {
            foreach (var value in returnedStructs) value.Dispose();
        }
        Assert(FfiModule.FunctionReturnPrimitive() == 42, "i32 return");
        Assert(Math.Abs(FfiModule.FunctionReturnFloat() - 5.21) < 0.000001, "f64 return");
        Assert(Math.Abs(FfiModule.FunctionReturnFloat32() - 4.20f) < 0.00001f, "f32 return");
        Assert(FfiModule.FunctionReturnNegatedBool(true) == false, "bool true argument");
        Assert(FfiModule.FunctionReturnNegatedBool(false) == true, "bool false argument");
        FfiModule.FunctionWithStringArg("Hello, 世界!");
        bool rejectedNul = false;
        try
        {
            FfiModule.FunctionWithStringArg("invalid\0string");
        }
        catch (ArgumentException)
        {
            rejectedNul = true;
        }
        Assert(rejectedNul, "NUL string argument rejection");
        Assert(
            FfiModule.FunctionReturnString() == "String returned from Rust: こんにちは",
            "UTF-8 string return");
        using (var returnedStruct = FfiModule.FunctionReturningStruct())
        {
            Assert(returnedStruct.I32Field == 48, "struct return");
        }
        using (var structArgument = TestStruct2.NewTs2(55))
        {
            FfiModule.FunctionTakingStruct(structArgument);
        }
        using (var comboInput = new TestStruct())
        {
            Assert(
                FfiModule.ComboFunction("first", "second", true, comboInput) == "first",
                "combined string and struct arguments");
            using (var second = new TestStruct())
            using (var third = TestStruct2.NewTs2(0))
            using (var comboResult = FfiModule.ComboStructFunction(comboInput, second, third))
            {
                Assert(comboResult.I32Field == 0, "combined struct arguments and return");
            }
        }
    }

    private static void ResultTests()
    {
        FfiModule.FunctionWithUnitExpressionResult();
        Assert(FfiModule.FunctionWithPrimitiveResult(false) == 123, "Result<int> success");
        bool primitiveResultFailed = false;
        try
        {
            FfiModule.FunctionWithPrimitiveResult(true);
        }
        catch (InvalidOperationException error)
        {
            primitiveResultFailed =
                error.Message == "StructError: EnumError: VariantTwo"
                && error.InnerException?.Message == "EnumError: VariantTwo"
                && error.InnerException.InnerException?.Message == "SimpleError"
                && error.InnerException.InnerException.InnerException is null;
        }
        Assert(primitiveResultFailed, "Result error source chain");
        Assert(FfiModule.FunctionWithBoolResult(false), "Result<bool> success");
        using (var resultStruct = FfiModule.FunctionWithStructResult(false))
        {
            Assert(resultStruct.I32Field == 256, "Result<struct> success");
        }
        bool resultFailed = false;
        try
        {
            FfiModule.FunctionWithBoolResult(true);
        }
        catch (InvalidOperationException error)
        {
            resultFailed = error.Message == "SimpleError";
        }
        Assert(resultFailed, "Result<bool> error conversion");
        Assert(
            FfiModule.FunctionWithStringResult(false) == "No error",
            "Result<string> success");
        Assert(
            FfiModule.FunctionWithEnumResult(false) == TestStatus.Pending,
            "Result<enum> success");
        Assert(
            FfiModule.FunctionWithVecStringResult(false).SequenceEqual(new[] { "One", "Two", "Three" }),
            "Result<vector<string>> success");
        Assert(
            FfiModule.FunctionWithVecIntResult(false).SequenceEqual(new[] { 10, 20, 30 }),
            "Result<vector<int>> success");
        Assert(
            FfiModule.FunctionWithVecBoolResult(false).SequenceEqual(new[] { true, false, false }),
            "Result<vector<bool>> success");
        var resultVectorStructs = FfiModule.FunctionWithVecStructResult(false);
        try
        {
            Assert(
                resultVectorStructs.Select(value => value.I32Field).SequenceEqual(new[] { 512, 1024 }),
                "Result<vector<struct>> success");
        }
        finally
        {
            foreach (var value in resultVectorStructs) value.Dispose();
        }
        Assert(
            FfiModule.FunctionWithVecEnumResult(false).SequenceEqual(
                new[] { TestStatus.Pending, TestStatus.Active, TestStatus.Inactive }),
            "Result<vector<enum>> success");
        Assert(
            FfiModule.FunctionWithHashmapResult(false)["key"] == "value",
            "Result<HashMap> success");
        foreach (var failingResult in new Action[]
        {
            () => FfiModule.FunctionWithStringResult(true),
            () => FfiModule.FunctionWithEnumResult(true),
            () => FfiModule.FunctionWithVecStringResult(true),
            () => FfiModule.FunctionWithVecEnumResult(true),
        })
        {
            bool failedWithRustError = false;
            try
            {
                failingResult();
            }
            catch (InvalidOperationException error)
            {
                failedWithRustError = error.Message == "EnumError: VariantOne";
            }
            Assert(failedWithRustError, "Result error description");
        }
        bool hashmapResultFailed = false;
        try
        {
            FfiModule.FunctionWithHashmapResult(true);
        }
        catch (InvalidOperationException error)
        {
            hashmapResultFailed = error.Message == "SimpleError";
        }
        Assert(hashmapResultFailed, "Result<HashMap> error");
    }

    private static void OptionAndEnumTests()
    {
        FfiModule.FunctionTakingSomeInt(10, true);
        FfiModule.FunctionTakingSomeInt(null, false);
        FfiModule.FunctionTakingSomeBool(true, true);
        FfiModule.FunctionTakingSomeBool(null, false);
        FfiModule.FunctionTakingSomeEnum(TestStatus.Pending, true);
        FfiModule.FunctionTakingSomeEnum(null, false);
        FfiModule.FunctionTakingSomeString("Some string", true);
        FfiModule.FunctionTakingSomeString(null, false);
        using (var optionalStruct = new TestStruct())
        {
            optionalStruct.I32Field = 567;
            FfiModule.FunctionTakingSomeStruct(optionalStruct, true);
        }
        FfiModule.FunctionTakingSomeStruct(null, false);
        Assert(FfiModule.FunctionReturningOptInt(true) == 100, "Some<int> return");
        Assert(FfiModule.FunctionReturningOptInt(false) is null, "None<int> return");
        Assert(FfiModule.FunctionReturningOptBool(true) == true, "Some<bool> return");
        Assert(FfiModule.FunctionReturningOptBool(false) is null, "None<bool> return");
        Assert(
            FfiModule.FunctionReturningOptEnum(true) == TestStatus.Pending,
            "Some<enum> return");
        Assert(FfiModule.FunctionReturningOptEnum(false) is null, "None<enum> return");
        Assert(
            FfiModule.FunctionReturningOptString(true) == "Some Rust String",
            "Some<string> return");
        Assert(FfiModule.FunctionReturningOptString(false) is null, "None<string> return");
        using (var optionalStruct = FfiModule.FunctionReturningOptStruct(true))
        {
            Assert(optionalStruct?.I32Field == 234, "Some<struct> return");
        }
        Assert(FfiModule.FunctionReturningOptStruct(false) is null, "None<struct> return");
        Assert(FfiModule.GetStatus() == TestStatus.Active, "enum return");
        Assert(FfiModule.GetInactiveStatus() == TestStatus.Inactive, "second enum return");
        FfiModule.AssertActive(TestStatus.Active);
        FfiModule.AssertInactive(TestStatus.Inactive);
    }

    private static void NestedFieldAndMethodTests()
    {
        using (var nestedStruct = new StructWithNestedContainers())
        {
            nestedStruct.VecOption = new List<int?> { 17, null, 19 };
            Assert(
                nestedStruct.VecOption.SequenceEqual(new int?[] { 17, null, 19 }),
                "Vec<Option<int>> struct field");
            Assert(nestedStruct.OptionVec is null, "default None nested struct field");
            nestedStruct.OptionVec = new List<int?> { 21, null, 23 };
            Assert(
                nestedStruct.OptionVec!.SequenceEqual(new int?[] { 21, null, 23 }),
                "Option<Vec<Option<int>>> struct field");
            nestedStruct.OptionVec = null;
            Assert(nestedStruct.OptionVec is null, "None nested struct field setter");
            nestedStruct.Deep = new List<IReadOnlyList<int?>?> {
                new List<int?> { 25, null }, null
            };
            Assert(
                nestedStruct.Deep[0]!.SequenceEqual(new int?[] { 25, null })
                    && nestedStruct.Deep[1] is null,
                "deep nested struct field");
            nestedStruct.TakeNested(new List<int?> { 7, null, 9 });
            Assert(
                nestedStruct.ReturnNested()!.SequenceEqual(new int?[] { 11, null, 13 }),
                "nested instance method return");
        }
        using (var methodTarget = new TestStruct())
        {
            methodTarget.PublicMethod();
            methodTarget.PublicMethodTakingPrimitives(10, false);
            methodTarget.PublicMethodTakingString("Test string from caller");
            using (var methodArgument = new TestStruct2())
            {
                methodArgument.I32Field = 55;
                methodTarget.PublicMethodTakingStruct(methodArgument);
            }
            methodTarget.PublicMethodTakingHashmap(
                new Dictionary<string, int> { ["method"] = 24 });
            using (var methodMapValue = new TestStruct2())
            {
                methodMapValue.I32Field = 24;
                methodTarget.PublicMethodTakingStructHashmap(
                    new Dictionary<string, TestStruct2> { ["method"] = methodMapValue });
            }
            methodTarget.PublicMethodTakingRecursiveMaps(
                new Dictionary<string, IReadOnlyList<int>>
                {
                    ["values"] = new List<int> { 1, 2 },
                },
                new Dictionary<string, int?> { ["some"] = 3, ["none"] = null },
                new Dictionary<string, Dictionary<string, int>>
                {
                    ["inner"] = new Dictionary<string, int> { ["value"] = 4 },
                },
                new List<Dictionary<string, int>> { new() { ["value"] = 5 } },
                new Dictionary<string, int> { ["value"] = 6 });
            Assert(methodTarget.PublicMethodReturningPrimitive() == 24, "instance method primitive");
            Assert(methodTarget.PublicMethodReturningBool(), "instance method bool");
            Assert(
                methodTarget.PublicMethodReturningString() == "String returned from Rust method",
                "instance method string");
            using (var returned = methodTarget.PublicMethodReturningStruct())
            {
                Assert(returned.I32Field == 99, "instance method struct");
            }
            Assert(
                methodTarget.PublicMethodReturningHashmap()["method"] == 25,
                "instance method HashMap return");
            methodTarget.PublicMethodTakingEnumHashmap(
                new Dictionary<TestStatus, TestStatus>
                {
                    [TestStatus.Active] = TestStatus.Active,
                    [TestStatus.Pending] = TestStatus.Inactive,
                });
            var methodEnumMap = methodTarget.PublicMethodReturningEnumHashmap();
            Assert(
                methodEnumMap[TestStatus.Active] == TestStatus.Pending
                    && methodEnumMap[TestStatus.Inactive] == TestStatus.Active,
                "instance method enum HashMap return");
            Assert(
                methodTarget.PublicMethodReturningVecOfHashmaps()[0]["value"] == 7,
                "instance method vector of HashMaps return");
            var methodStructMap = methodTarget.PublicMethodReturningStructHashmap();
            try
            {
                Assert(methodStructMap["method"].I32Field == 25, "instance method struct HashMap");
            }
            finally
            {
                foreach (var value in methodStructMap.Values) value.Dispose();
            }
            Assert(
                methodTarget.ComboMethod("first", "second", true) == "first",
                "instance method string arguments");
            methodTarget.PublicMethodTakingVecOfPrimitives(new List<int> { 1, 2, 3, 4, 5 });
            methodTarget.PublicMethodTakingVecOfStrings(new List<string> { "Hello", "World" });
            using (var first = new TestStruct2())
            using (var second = new TestStruct2())
            {
                first.I32Field = 42;
                second.I32Field = 24;
                methodTarget.PublicMethodTakingVecOfStructs(new List<TestStruct2> { first, second });
            }
            Assert(
                methodTarget.PublicMethodReturningVecOfPrimitives().SequenceEqual(new[] { 10, 20, 30, 40, 50 }),
                "instance method integer vector");
            Assert(
                methodTarget.PublicMethodReturningVecOfStrings().SequenceEqual(new[] { "Method", "Vector", "Return" }),
                "instance method string vector");
            var methodStructs = methodTarget.PublicMethodReturningVecOfStructs();
            try
            {
                Assert(
                    methodStructs.Select(value => value.I32Field).SequenceEqual(new[] { 300, 400 }),
                    "instance method struct vector");
            }
            finally
            {
                foreach (var value in methodStructs) value.Dispose();
            }
            methodTarget.MethodTakingOptInt(20, true);
            methodTarget.MethodTakingOptInt(null, false);
            TestStruct.StaticMethodTakingOptString("Optional string", true);
            TestStruct.StaticMethodTakingOptString(null, false);
            TestStruct.StaticMethodTakingOptEnum(TestStatus.Active, true);
            TestStruct.StaticMethodTakingOptEnum(null, false);
            methodTarget.MethodTakingOptBool(false, true);
            methodTarget.MethodTakingOptBool(null, false);
            using (var optionalMethodStruct = new TestStruct2())
            {
                optionalMethodStruct.I32Field = 789;
                methodTarget.MethodTakingOptStruct(optionalMethodStruct, true);
            }
            methodTarget.MethodTakingOptStruct(null, false);
            Assert(methodTarget.MethodReturningOptInt(true) == 30, "instance method Some<int>");
            Assert(methodTarget.MethodReturningOptInt(false) is null, "instance method None<int>");
            Assert(
                TestStruct.StaticMethodReturningOptBool(true) == false,
                "static method Some<bool>");
            Assert(TestStruct.StaticMethodReturningOptBool(false) is null, "static method None<bool>");
            Assert(
                TestStruct.StaticMethodReturningOptEnum(true) == TestStatus.Inactive,
                "static method Some<enum>");
            Assert(TestStruct.StaticMethodReturningOptEnum(false) is null, "static method None<enum>");
            Assert(
                methodTarget.MethodReturningOptString(true) == "Optional string from Rust",
                "instance method Some<string>");
            Assert(methodTarget.MethodReturningOptString(false) is null, "instance method None<string>");
            using (var optionalReturned = methodTarget.MethodReturningOptStruct(true))
            {
                Assert(optionalReturned?.I32Field == 654, "instance method Some<struct>");
            }
            Assert(methodTarget.MethodReturningOptStruct(false) is null, "instance method None<struct>");
            Assert(methodTarget.MethodWithStringResult() == "Ok!", "Result<string> from instance method");
            Assert(methodTarget.MethodWithIntResult(false) == 16, "Result<i64> from instance method");
            bool methodIntResultFailed = false;
            try
            {
                methodTarget.MethodWithIntResult(true);
            }
            catch (InvalidOperationException error)
            {
                methodIntResultFailed =
                    error.Message == "StructError: EnumError: VariantTwo"
                    && error.InnerException?.Message == "EnumError: VariantTwo"
                    && error.InnerException.InnerException?.Message == "SimpleError";
            }
            Assert(methodIntResultFailed, "Result error source chain from instance method");
            Assert(
                methodTarget.MethodWithEnumResult() == TestStatus.Pending,
                "Result<enum> from instance method");
            Assert(
                methodTarget.MethodReturningResultWithHashmap(false)["key"] == 42,
                "Result<HashMap> from instance method");
            bool methodHashmapResultFailed = false;
            try
            {
                methodTarget.MethodReturningResultWithHashmap(true);
            }
            catch (InvalidOperationException error)
            {
                methodHashmapResultFailed = error.Message == "SimpleError";
            }
            Assert(methodHashmapResultFailed, "Result<HashMap> method error");
            Assert(
                TestStruct.StaticMethodWithBoolResult(false),
                "Result<bool> from static method");
            using (var resultStruct = TestStruct.StaticMethodWithStructResult())
            {
                Assert(resultStruct.I32Field == 267, "Result<struct> from static method");
            }
            using (var statusStruct = StructWithStatus.Inactivate())
            {
                Assert(statusStruct.Status == TestStatus.Inactive, "enum struct field");
                statusStruct.Status = TestStatus.Active;
                Assert(statusStruct.Status == TestStatus.Active, "enum struct field setter");
            }
            bool methodResultFailed = false;
            try
            {
                TestStruct.StaticMethodWithBoolResult(true);
            }
            catch (InvalidOperationException error)
            {
                methodResultFailed = error.Message == "SimpleError";
            }
            Assert(methodResultFailed, "Result error from static method");
            Assert(
                methodTarget.MethodWithVecOfIntsResult().SequenceEqual(new sbyte[] { 1, 2, 7 }),
                "Result<vector<int>> from instance method");
            Assert(
                TestStruct.StaticMethodWithVecOfBoolsResult().SequenceEqual(new[] { true, false, false }),
                "Result<vector<bool>> from static method");
            Assert(
                TestStruct.StaticMethodWithVecOfEnumResult().SequenceEqual(
                    new[] { TestStatus.Inactive, TestStatus.Pending, TestStatus.Active }),
                "Result<vector<enum>> from static method");
            Assert(
                methodTarget.MethodWithVecOfStringsResult().SequenceEqual(new[] { "some", "string" }),
                "Result<vector<string>> from instance method");
            Assert(
                methodTarget.MethodWithVecOfBoolsResult().SequenceEqual(new[] { false, true, true, true }),
                "Result<vector<bool>> from instance method");
            var resultStructs = TestStruct.StaticMethodWithVecOfStructsResult();
            try
            {
                Assert(
                    resultStructs.Select(value => value.I32Field).SequenceEqual(new[] { 2, -5 }),
                    "Result<vector<struct>> from static method");
            }
            finally
            {
                foreach (var value in resultStructs) value.Dispose();
            }
        }
        TestStruct.StaticMethod();
        TestStruct.StaticMethodTakingPrimitives(20, true);
        TestStruct.StaticMethodTakingString("static method string");
        using (var staticArgument = new TestStruct2())
        {
            staticArgument.I32Field = 77;
            TestStruct.StaticMethodTakingStruct(staticArgument);
        }
        TestStruct.StaticMethodTakingHashmap(
            new Dictionary<string, int> { ["static"] = 77 });
        using (var staticMapValue = new TestStruct2())
        {
            staticMapValue.I32Field = 77;
            TestStruct.StaticMethodTakingStructHashmap(
                new Dictionary<string, TestStruct2> { ["static"] = staticMapValue });
        }
        Assert(TestStruct.StaticMethodReturningPrimitive() == 22, "static method primitive");
        Assert(
            TestStruct.StaticMethodReturningString() == "String returned from Rust static method",
            "static method string");
        using (var staticReturned = TestStruct.StaticMethodReturningStruct())
        {
            Assert(staticReturned.I32Field == 77, "static method struct");
        }
        Assert(
            TestStruct.StaticMethodReturningHashmap()["static"] == 78,
            "static method HashMap return");
        var staticStructMap = TestStruct.StaticMethodReturningStructHashmap();
        try
        {
            Assert(staticStructMap["static"].I32Field == 78, "static method struct HashMap");
        }
        finally
        {
            foreach (var value in staticStructMap.Values) value.Dispose();
        }
        Assert(TestStruct.StaticComboMethod("first", "second", false) == "second", "static combo method");
        using (var comboFirst = TestStruct2.NewTs2(8))
        using (var comboSecond = TestStruct2.NewTs2(9))
        using (var comboStruct = TestStruct.StaticComboStructMethod(comboFirst, comboSecond))
        {
            Assert(comboStruct.I32Field == 8, "static method with struct arguments");
        }
        TestStruct.StaticMethodTakingEnumHashmap(
            new Dictionary<TestStatus, TestStatus>
            {
                [TestStatus.Active] = TestStatus.Inactive,
                [TestStatus.Pending] = TestStatus.Pending,
            });
        var staticEnumMap = TestStruct.StaticMethodReturningEnumHashmap();
        Assert(
            staticEnumMap[TestStatus.Inactive] == TestStatus.Pending
                && staticEnumMap[TestStatus.Pending] == TestStatus.Active,
            "static method enum HashMap return");
        using (var defaultStruct3 = new TestStruct3())
        using (var defaultStruct4 = new TestStruct4())
        using (var returnedStruct4 = TestStruct4.StaticMethodReturningSelf(123))
        {
            Assert(defaultStruct3.I32Field == 42, "custom Default impl after struct");
            Assert(defaultStruct4.I32Field == 99, "custom Default impl before struct");
            Assert(returnedStruct4.I32Field == 123, "static method returning its struct");
        }
        TestStruct.StaticMethodTakingVecOfPrimitives(new List<ushort> { 6, 7, 8, 9, 10 });
        TestStruct.StaticMethodTakingVecOfStrings(new List<string> { "Static", "Method" });
        using (var first = new TestStruct2())
        using (var second = new TestStruct2())
        {
            first.I32Field = 100;
            second.I32Field = 200;
            TestStruct.StaticMethodTakingVecOfStructs(new List<TestStruct2> { first, second });
        }
        Assert(
            TestStruct.StaticMethodReturningVecOfPrimitives().SequenceEqual(new[] { 60, 70, 80, 90, 100 }),
            "static method integer vector");
        Assert(
            TestStruct.StaticMethodReturningVecOfStrings().SequenceEqual(new[] { "Static", "Method", "Vector" }),
            "static method string vector");
        var staticStructs = TestStruct.StaticMethodReturningVecOfStructs();
        try
        {
            Assert(
                staticStructs.Select(value => value.I32Field).SequenceEqual(new[] { 500, 600 }),
                "static method struct vector");
        }
        finally
        {
            foreach (var value in staticStructs) value.Dispose();
        }
        using (var vectors = new StructWithVecs())
        {
            Assert(vectors.VecOfInts.Count == 0, "default integer vector field");
            vectors.VecOfInts = new List<int> { 3, 5, 7 };
            Assert(vectors.VecOfInts.SequenceEqual(new[] { 3, 5, 7 }), "integer vector field getter/setter");
            vectors.VecOfBools = new List<bool> { true, false, true };
            Assert(vectors.VecOfBools.SequenceEqual(new[] { true, false, true }), "bool vector field getter/setter");
            vectors.VecOfStrings = new List<string> { "new", "vec" };
            Assert(vectors.VecOfStrings.SequenceEqual(new[] { "new", "vec" }), "string vector field getter/setter");
            using (var fieldStruct = new TestStruct2())
            {
                fieldStruct.I32Field = 567;
                vectors.VecOfStructs = new List<TestStruct2> { new TestStruct2(), fieldStruct };
            }
            var vectorStructs = vectors.VecOfStructs;
            try
            {
                Assert(
                    vectorStructs.Select(value => value.I32Field).SequenceEqual(new[] { 0, 567 }),
                    "struct vector field getter/setter");
            }
            finally
            {
                foreach (var value in vectorStructs) value.Dispose();
            }
            vectors.VecOfEnums = new List<TestStatus> { TestStatus.Active, TestStatus.Pending };
            Assert(
                vectors.VecOfEnums.SequenceEqual(new[] { TestStatus.Active, TestStatus.Pending }),
                "enum vector field getter/setter");
        }
        using (var options = new StructWithOptions())
        {
            Assert(options.OptInt is null, "default optional integer field");
            options.OptInt = 42;
            Assert(options.OptInt == 42, "optional integer field getter/setter");
            options.OptInt = null;
            Assert(options.OptInt is null, "optional integer field None setter");
            options.OptBool = true;
            Assert(options.OptBool == true, "optional bool field getter/setter");
            options.OptString = "field option";
            Assert(options.OptString == "field option", "optional string field getter/setter");
            options.OptEnum = TestStatus.Inactive;
            Assert(options.OptEnum == TestStatus.Inactive, "optional enum field getter/setter");
            using (var optionStruct = new TestStruct2())
            {
                optionStruct.I32Field = 123;
                options.OptStruct = optionStruct;
            }
            using (var returnedOptionStruct = options.OptStruct)
            {
                Assert(returnedOptionStruct?.I32Field == 123, "optional struct field getter/setter");
            }
            options.OptStruct = null;
            Assert(options.OptStruct is null, "optional struct field None setter");
        }
        using (var defaultStruct = new TestStruct())
        {
            Assert(defaultStruct.I32Field == 0, "default struct constructor");
            defaultStruct.I32Field = 17;
            Assert(defaultStruct.I32Field == 17, "struct property getter/setter");
            Assert(defaultStruct.StringField == string.Empty, "default public string field");
            defaultStruct.StringField = "field value";
            Assert(defaultStruct.StringField == "field value", "public string field property");
            Assert(!defaultStruct.BoolField, "default bool field");
            defaultStruct.BoolField = true;
            Assert(defaultStruct.BoolField, "bool field property getter/setter");
            Assert(defaultStruct.NoSetter == 0, "getter-only field");
            Assert(Math.Abs(defaultStruct.F32Field) < 0.00001f, "public f32 getter");
        }
    }

    private static void TraitTests()
    {
        var implementation = DispatchProxy.Create<RustTrait, RustTraitTestProxy>();
        FfiModule.FunctionTakingTraitObject(implementation);
        Assert(implementation.TraitFnReturnInt() == 12345, "managed trait remains usable");

        var concreteImplementation = new RustTraitTestImplementation();
        FfiModule.FunctionTakingTraitObject(concreteImplementation);
        Assert(
            concreteImplementation.TraitFnReturnString() == "String from trait object",
            "concrete managed trait remains usable");

        using (var receiver = new TestStruct())
        {
            receiver.MethodTakingTraitObject(implementation);
            TestStruct.StaticMethodTakingTraitObject(implementation);
            using (var returned = receiver.MethodReturningTraitObject())
            {
                AssertRustTraitObject(returned);
                FfiModule.FunctionTakingTraitObject(returned);
            }
            using (var returned = TestStruct.StaticMethodReturningTraitObject())
            {
                AssertRustTraitObject(returned);
                FfiModule.FunctionTakingTraitObject(returned);
            }
        }

        using (var returned = FfiModule.FunctionReturningTraitObject())
        {
            AssertRustTraitObject(returned);
            FfiModule.FunctionTakingTraitObject(returned);
        }
    }

    private static void AssertRustTraitObject(RustTrait trait)
    {
        trait.TraitSimpleFn();
        trait.TraitFnWithSimpleArgs(42, 4.2, TestStatus.Pending, true);
        trait.TraitFnWithStringArg("Hello from trait object");
        using (var input = new TestStruct())
        {
            input.I32Field = 123;
            trait.TraitFnWithStructArg(input);
        }
        trait.TraitFnWithVecOfPrimitives(new[] { 1, 2, 3, 4, 5 });
        trait.TraitFnWithVecOfBools(new[] { true, false, true });
        trait.TraitFnWithVecOfEnums(new[] { TestStatus.Active, TestStatus.Inactive });
        trait.TraitFnWithVecOfStrings(new[] { "Hello", "Trait", "Object" });
        using (var first = new TestStruct2())
        using (var second = new TestStruct2())
        {
            first.I32Field = 321;
            second.I32Field = 654;
            trait.TraitFnWithVecOfStructs(new[] { first, second });
        }
        trait.TraitFnWithIntHashmap(new Dictionary<int, int> { [1] = 10, [2] = 20 });
        trait.TraitFnWithStringHashmap(
            new Dictionary<string, string> { ["alpha"] = "one", ["beta"] = "two" });
        trait.TraitFnWithEnumHashmap(
            new Dictionary<TestStatus, TestStatus>
            {
                [TestStatus.Active] = TestStatus.Pending,
                [TestStatus.Inactive] = TestStatus.Active,
            });
        trait.TraitFnWithRecursiveMaps(
            new Dictionary<string, IReadOnlyList<int>> { ["values"] = new[] { 1, 2 } },
            new Dictionary<string, int?> { ["some"] = 3, ["none"] = null },
            new Dictionary<string, Dictionary<string, int>>
            {
                ["inner"] = new() { ["value"] = 4 },
            },
            new[] { new Dictionary<string, int> { ["value"] = 5 } },
            new Dictionary<string, int> { ["value"] = 6 });
        using (var alpha = new TestStruct2())
        using (var beta = new TestStruct2())
        {
            alpha.I32Field = 10;
            beta.I32Field = 20;
            trait.TraitFnWithStructHashmap(
                new Dictionary<string, TestStruct2> { ["alpha"] = alpha, ["beta"] = beta });
        }
        using (var optionalStruct = new TestStruct2())
        {
            optionalStruct.I32Field = 789;
            trait.TraitFnWithOptions(
                42, "Hello from trait object", false, TestStatus.Pending, optionalStruct);
        }

        Assert(trait.TraitFnReturnInt() == 12345, "trait primitive return");
        Assert(trait.TraitFnReturnBool(), "trait bool return");
        Assert(trait.TraitFnReturnString() == "String from trait object", "trait string return");
        using (var returned = trait.TraitFnReturnStruct())
        {
            Assert(returned.I32Field == 987, "trait struct return");
        }
        Assert(trait.TraitFnReturnEnum() == TestStatus.Inactive, "trait enum return");
        Assert(trait.TraitFnReturnVecOfPrimitives().SequenceEqual(new[] { 10, 20, 30 }), "trait vector return");
        Assert(
            trait.TraitFnReturnVecOfBools().SequenceEqual(new[] { true, false, true, true }),
            "trait bool vector return");
        Assert(
            trait.TraitFnReturnVecOfEnums().SequenceEqual(
                new[] { TestStatus.Active, TestStatus.Inactive, TestStatus.Pending }),
            "trait enum vector return");
        Assert(
            trait.TraitFnReturnVecOfStrings().SequenceEqual(new[] { "Hello", "from", "trait", "object" }),
            "trait string vector return");
        var returnedStructs = trait.TraitFnReturnVecOfStructs();
        try
        {
            Assert(
                returnedStructs.Select(value => value.I32Field).SequenceEqual(new[] { 111, 222 }),
                "trait struct vector return");
        }
        finally
        {
            foreach (var value in returnedStructs) value.Dispose();
        }
        var intMap = trait.TraitFnReturnIntHashmap();
        Assert(intMap.Count == 2 && intMap[3] == 30 && intMap[4] == 40, "trait integer HashMap return");
        var stringMap = trait.TraitFnReturnStringHashmap();
        Assert(stringMap["gamma"] == "three" && stringMap["delta"] == "four", "trait string HashMap return");
        var enumMap = trait.TraitFnReturnEnumHashmap();
        Assert(
            enumMap[TestStatus.Active] == TestStatus.Inactive
                && enumMap[TestStatus.Pending] == TestStatus.Active,
            "trait enum HashMap return");
        var structMap = trait.TraitFnReturnStructHashmap();
        try
        {
            Assert(
                structMap["alpha"].I32Field == 30 && structMap["beta"].I32Field == 40,
                "trait struct HashMap return");
        }
        finally
        {
            foreach (var value in structMap.Values) value.Dispose();
        }
        Assert(
            trait.TraitFnReturnMapOfVec()["values"].SequenceEqual(new[] { 16, 17 }),
            "trait map of vectors return");

        Assert(trait.TraitFnReturningOptionInt(true) == 555, "trait Some<int>");
        Assert(trait.TraitFnReturningOptionInt(false) is null, "trait None<int>");
        Assert(trait.TraitFnReturningOptionBool(true) == false, "trait Some<bool>");
        Assert(trait.TraitFnReturningOptionBool(false) is null, "trait None<bool>");
        Assert(trait.TraitFnReturningOptionString(true) == "Some string", "trait Some<string>");
        Assert(trait.TraitFnReturningOptionString(false) is null, "trait None<string>");
        Assert(trait.TraitFnReturningOptionEnum(true) == TestStatus.Pending, "trait Some<enum>");
        Assert(trait.TraitFnReturningOptionEnum(false) is null, "trait None<enum>");
        using (var value = trait.TraitFnReturningOptionStruct(true))
        {
            Assert(value?.I32Field == 789, "trait Some<struct>");
        }
        Assert(trait.TraitFnReturningOptionStruct(false) is null, "trait None<struct>");

        Assert(trait.TraitFnReturningResultInt(false) == 555, "trait Result<int> success");
        AssertFailure(
            () => trait.TraitFnReturningResultInt(true),
            "StructError: EnumError: VariantTwo",
            "trait Result<int> error");
        Assert(trait.TraitFnReturningResultBool(false), "trait Result<bool> success");
        AssertFailure(() => trait.TraitFnReturningResultBool(true), "SimpleError", "trait Result<bool> error");
        Assert(
            trait.TraitFnReturningResultString(false) == "Some string",
            "trait Result<string> success");
        AssertFailure(
            () => trait.TraitFnReturningResultString(true),
            "SimpleError",
            "trait Result<string> error");
        using (var value = trait.TraitFnReturningResultStruct(false))
        {
            Assert(value.I32Field == 789, "trait Result<struct> success");
        }
        AssertFailure(
            () => trait.TraitFnReturningResultStruct(true),
            "Error from trait object",
            "trait Result<struct> error");
        Assert(trait.TraitFnReturningResultEnum(false) == TestStatus.Active, "trait Result<enum> success");
        AssertFailure(
            () => trait.TraitFnReturningResultEnum(true),
            "Error from trait object",
            "trait Result<enum> error");
        Assert(
            trait.TraitFnReturningResultVecOfInts(false).SequenceEqual(new[] { 1, 2, 3 }),
            "trait Result<vector<int>> success");
        AssertFailure(
            () => trait.TraitFnReturningResultVecOfInts(true),
            "Error from trait object",
            "trait Result<vector<int>> error");
        Assert(
            trait.TraitFnReturningResultVecOfBools(false).SequenceEqual(new[] { true, false, true }),
            "trait Result<vector<bool>> success");
        AssertFailure(
            () => trait.TraitFnReturningResultVecOfBools(true),
            "Error from trait object",
            "trait Result<vector<bool>> error");
        Assert(
            trait.TraitFnReturningResultVecOfEnums(false).SequenceEqual(
                new[] { TestStatus.Active, TestStatus.Inactive, TestStatus.Pending }),
            "trait Result<vector<enum>> success");
        AssertFailure(
            () => trait.TraitFnReturningResultVecOfEnums(true),
            "Error from trait object",
            "trait Result<vector<enum>> error");
        Assert(
            trait.TraitFnReturningResultVecOfStrings(false).SequenceEqual(
                new[] { "Hello", "from", "trait", "object" }),
            "trait Result<vector<string>> success");
        AssertFailure(
            () => trait.TraitFnReturningResultVecOfStrings(true),
            "Error from trait object",
            "trait Result<vector<string>> error");
        var resultStructs = trait.TraitFnReturningResultVecOfStructs(false);
        try
        {
            Assert(
                resultStructs.Select(value => value.I32Field).SequenceEqual(new[] { 111, 222 }),
                "trait Result<vector<struct>> success");
        }
        finally
        {
            foreach (var value in resultStructs) value.Dispose();
        }
        AssertFailure(
            () => trait.TraitFnReturningResultVecOfStructs(true),
            "Error from trait object",
            "trait Result<vector<struct>> error");
        Assert(trait.TraitFnReturningResultWithOptionInt(false) == 555, "trait Result<Option<int>> success");
        AssertFailure(
            () => trait.TraitFnReturningResultWithOptionInt(true),
            "Error from trait object",
            "trait Result<Option<int>> error");
        Assert(
            trait.TraitFnReturningResultWithOptionBool(false) == false,
            "trait Result<Option<bool>> success");
        AssertFailure(
            () => trait.TraitFnReturningResultWithOptionBool(true),
            "Error from trait object",
            "trait Result<Option<bool>> error");
        Assert(
            trait.TraitFnReturningResultWithOptionString(false) == "Some string",
            "trait Result<Option<string>> success");
        AssertFailure(
            () => trait.TraitFnReturningResultWithOptionString(true),
            "Error from trait object",
            "trait Result<Option<string>> error");
        Assert(
            trait.TraitFnReturningResultWithOptionEnum(false) == TestStatus.Pending,
            "trait Result<Option<enum>> success");
        AssertFailure(
            () => trait.TraitFnReturningResultWithOptionEnum(true),
            "Error from trait object",
            "trait Result<Option<enum>> error");
        using (var value = trait.TraitFnReturningResultWithOptionStruct(false))
        {
            Assert(value?.I32Field == 789, "trait Result<Option<struct>> success");
        }
        AssertFailure(
            () => trait.TraitFnReturningResultWithOptionStruct(true),
            "Error from trait object",
            "trait Result<Option<struct>> error");
        trait.TraitFnReturningResultWithUnitExpression(false);
        AssertFailure(
            () => trait.TraitFnReturningResultWithUnitExpression(true),
            "SimpleError",
            "trait Result<()> error");
        var resultMap = trait.TraitFnReturningResultWithHashmap(false);
        try
        {
            Assert(
                resultMap["alpha"].I32Field == 30 && resultMap["beta"].I32Field == 40,
                "trait Result<HashMap> success");
        }
        finally
        {
            foreach (var value in resultMap.Values) value.Dispose();
        }
        AssertFailure(
            () => trait.TraitFnReturningResultWithHashmap(true),
            "SimpleError",
            "trait Result<HashMap> error");
    }

    private static void AssertFailure(Action action, string expectedMessage, string test)
    {
        try
        {
            action();
        }
        catch (InvalidOperationException error)
        {
            Assert(error.Message == expectedMessage, test);
            return;
        }
        Assert(false, test);
    }

    private static void Assert(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException($"Failed C# integration test: {message}");
        }
    }
}
