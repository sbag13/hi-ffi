use crate::wrapper::enum_wrapper::EnumWrapper;
use crate::wrapper::impl_block_wrapper::ImplBlockWrapper;
use crate::wrapper::java::{
    java_argument_type, java_layout, java_library_loader, java_option_class_name,
    java_result_class_name, java_return_type, java_vec_class_name, rust_exception_java,
};
use crate::wrapper::trait_wrapper::TraitWrapper;
use crate::wrapper::{ParsedWrapper, StructWrapper, Wrapper, WrapperType};
use crate::{
    EXPORTED_SYMBOLS_PREFIX, GEN_CODE_DIR, ReusableWrapper, create_file,
    prepend_each_line_with_n_tabs,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, Once};

const JAVA_CODE_DIR: &str = "java/";
static JAVA_MODULE_CREATED: Once = Once::new();
static JAVA_BASE_CREATED: Once = Once::new();
static JAVA_REUSABLE_WRAPPER_GENERATED: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
const EXTERNAL_METHOD_MARKER: &str = "// external methods marker";
const LINKER_INITIALIZATION_MARKER: &str = "// linker initialization marker";

pub(crate) fn write_java_code(wrapper: &Wrapper) {
    JAVA_BASE_CREATED.call_once(|| {
        create_file(java_library_loader(), java_path().join("HiFfiLibrary.java"));
        create_file(
            rust_exception_java(),
            java_path().join("RustException.java"),
        );
    });

    match &wrapper.parsed {
        ParsedWrapper::Struct(struct_wrapper) => write_struct_code(struct_wrapper),
        ParsedWrapper::Enum(enum_wrapper) => write_enum_code(enum_wrapper),
        ParsedWrapper::Function(function_wrapper) => write_function_code(function_wrapper),
        ParsedWrapper::ImplBlock(impl_wrapper) => write_impl_code(impl_wrapper),
        ParsedWrapper::Trait(trait_wrapper) => write_trait_code(trait_wrapper),
    }

    write_reusable_wrappers(wrapper);
}

fn write_reusable_wrappers(wrapper: &Wrapper) {
    for reusable_wrapper in &wrapper.reusable_wrappers {
        let file_name = match reusable_wrapper {
            ReusableWrapper::Vec(inner) => format!("vec_{}.java", inner.name()),
            ReusableWrapper::Result(inner) => format!("result_{}.java", inner.name()),
            ReusableWrapper::Option(inner) => format!("option_{}.java", inner.name()),
        };
        let path = java_path().join(file_name);
        if JAVA_REUSABLE_WRAPPER_GENERATED
            .lock()
            .expect("Mutex lock failed during locking for reusable wrapper")
            .insert(path.clone())
        {
            let Some(source) = reusable_wrapper.java() else {
                continue;
            };
            create_file(source, path);
        }
    }
}

fn insert_impl_methods(path: &Path, methods: &str) {
    if methods.is_empty() {
        return;
    }
    let mut source = std::fs::read_to_string(path).expect("Unable to read Java struct");
    let marker = "    @Override\n    public void close()";
    source = source.replace(marker, &format!("{methods}\n{marker}"));
    if methods.contains("List<") && !source.contains("import java.util.List;") {
        source = format!("import java.util.List;\n{source}");
    }
    if methods.contains("Arena.ofConfined") && !source.contains("import java.lang.foreign.Arena;") {
        source = format!("import java.lang.foreign.Arena;\n{source}");
    }
    if methods.contains("Optional<") && !source.contains("import java.util.Optional;") {
        source = format!("import java.util.Optional;\n{source}");
    }
    create_file(source, path);
}

fn write_impl_code(impl_wrapper: &ImplBlockWrapper) {
    let path = java_path().join(format!("{}.java", impl_wrapper.struct_name));
    // Impl blocks defined BEFORE the struct are deferred and replayed only after the
    // struct is registered (see WAITING_FOR_WRAPPERS in lib.rs). Struct files are
    // written before deferred impls are replayed, so the file normally exists here.
    if !path.exists() {
        panic!(
            "hi-ffi [java]: struct file for `{}` not found yet, skipping impl block",
            impl_wrapper.struct_name
        );
    }

    insert_impl_methods(&path, &impl_methods_source(impl_wrapper));
    if let Some(default_constructor) = &impl_wrapper.default_constructor {
        insert_default_ctor(
            &path,
            &impl_wrapper.struct_name.to_string(),
            &default_constructor.extern_fn_name,
        );
    }
    if let Some(partial_eq) = &impl_wrapper.partial_eq {
        insert_partial_eq(
            &path,
            &impl_wrapper.struct_name.to_string(),
            &partial_eq.extern_fn_name,
        );
    }
}

fn write_trait_code(trait_wrapper: &TraitWrapper) {
    let trait_name = trait_wrapper.name.to_string();
    let methods = trait_wrapper
        .functions
        .iter()
        .map(trait_interface_method)
        .collect::<String>();
    let imports = trait_imports(&trait_wrapper.functions);
    let interface_source = format!(
        r#"{imports}public interface {trait_name} extends AutoCloseable {{
{methods}
    default void close() {{}}
}}
"#
    );
    create_file(
        interface_source,
        java_path().join(format!("{trait_name}.java")),
    );
    create_file(
        trait_bridge_source(trait_wrapper),
        java_path().join(format!("{trait_name}Bridge.java")),
    );
    create_file(
        trait_object_source(trait_wrapper),
        java_path().join(format!("{trait_name}Object.java")),
    );
}

fn trait_imports(functions: &[crate::wrapper::FunctionWrapper]) -> String {
    let mut imports = String::new();
    let wrappers = functions
        .iter()
        .flat_map(|function| {
            function
                .args
                .iter()
                .map(|arg| &arg.wrapper_type)
                .chain(function.return_wrapper.iter().map(|ret| &ret.wrapper_type))
        })
        .collect::<Vec<_>>();
    if wrappers.iter().any(|wrapper| wrapper_contains_vec(wrapper)) {
        imports.push_str("import java.util.List;\n");
    }
    if wrappers
        .iter()
        .any(|wrapper| wrapper_contains_option(wrapper))
    {
        imports.push_str("import java.util.Optional;\n");
    }
    imports
}

fn wrapper_contains_vec(wrapper_type: &WrapperType) -> bool {
    match wrapper_type {
        WrapperType::Vec(_) => true,
        WrapperType::Option(inner) | WrapperType::Result(inner) => wrapper_contains_vec(inner),
        _ => false,
    }
}

fn wrapper_contains_option(wrapper_type: &WrapperType) -> bool {
    match wrapper_type {
        WrapperType::Option(_) => true,
        WrapperType::Vec(inner) | WrapperType::Result(inner) => wrapper_contains_option(inner),
        _ => false,
    }
}

fn trait_interface_method(function: &crate::wrapper::FunctionWrapper) -> String {
    let return_type = function
        .return_wrapper
        .as_ref()
        .map(|ret| java_return_type(&ret.wrapper_type))
        .unwrap_or_else(|| Some("void".to_string()))
        .expect("Java trait return type");
    let parameters = function
        .args
        .iter()
        .map(|arg| {
            format!(
                "{} {}",
                java_argument_type(&arg.wrapper_type).expect("Java trait argument type"),
                arg.arg_name
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "    {return_type} {}({parameters});\n",
        to_java_method_name(&function.name.to_string())
    )
}

fn trait_callback_layout(wrapper_type: &WrapperType) -> String {
    match wrapper_type {
        WrapperType::IntegerNumber(_)
        | WrapperType::FloatingPointNumber(_)
        | WrapperType::Bool
        | WrapperType::Enum(_) => java_layout(wrapper_type),
        _ => "ValueLayout.ADDRESS".to_string(),
    }
}

fn trait_callback_carrier(wrapper_type: &WrapperType) -> &'static str {
    match wrapper_type {
        WrapperType::IntegerNumber(inner) => match inner.as_str() {
            "i8" | "u8" => "byte",
            "i16" | "u16" => "short",
            "i32" | "u32" => "int",
            "i64" | "u64" | "usize" => "long",
            _ => unreachable!(),
        },
        WrapperType::FloatingPointNumber(inner) => match inner.as_str() {
            "f32" => "float",
            "f64" => "double",
            _ => unreachable!(),
        },
        WrapperType::Bool => "boolean",
        WrapperType::Enum(_) => "int",
        _ => "MemorySegment",
    }
}

fn trait_callback_return(function: &crate::wrapper::FunctionWrapper) -> Option<&WrapperType> {
    function.return_wrapper.as_ref().and_then(|ret| {
        (!matches!(ret.wrapper_type, WrapperType::UnitExpr)).then_some(&ret.wrapper_type)
    })
}

fn trait_callback_descriptor(function: &crate::wrapper::FunctionWrapper) -> String {
    let mut layouts = vec!["ValueLayout.ADDRESS".to_string()];
    layouts.extend(
        function
            .args
            .iter()
            .map(|arg| trait_callback_layout(&arg.wrapper_type)),
    );
    function
        .return_wrapper
        .as_ref()
        .and_then(|ret| {
            (!matches!(ret.wrapper_type, WrapperType::UnitExpr))
                .then(|| java_layout(&ret.wrapper_type))
        })
        .map(|result| format!("FunctionDescriptor.of({result}, {})", layouts.join(", ")))
        .unwrap_or_else(|| format!("FunctionDescriptor.ofVoid({})", layouts.join(", ")))
}

fn trait_callback_method_type(function: &crate::wrapper::FunctionWrapper) -> String {
    let result = trait_callback_return(function)
        .map(trait_callback_carrier)
        .unwrap_or("void");
    let mut arguments = vec!["MemorySegment.class".to_string()];
    arguments.extend(
        function
            .args
            .iter()
            .map(|arg| format!("{}.class", trait_callback_carrier(&arg.wrapper_type))),
    );
    format!(
        "MethodType.methodType({result}.class, {})",
        arguments.join(", ")
    )
}

fn trait_callback_arg_conversion(
    arg: &crate::wrapper::FunctionArgWrapper,
) -> (String, Option<String>) {
    let name = arg.arg_name.to_string();
    let value_name = format!("{name}Value");
    match &arg.wrapper_type {
        WrapperType::String => (format!("var {value_name} = readString({name});\n"), None),
        WrapperType::Struct(struct_name) => (
            format!("var {value_name} = {struct_name}.fromRaw({name});\n"),
            Some(value_name),
        ),
        WrapperType::Vec(inner) => (
            format!(
                "var {name}Wrapper = {}.fromRaw({name});\nvar {value_name} = {name}Wrapper.toList();\n",
                java_vec_class_name(inner)
            ),
            Some(format!("{name}Wrapper")),
        ),
        WrapperType::Option(inner) => (
            format!(
                "var {name}Wrapper = {}.fromRaw({name});\nvar {value_name} = {name}Wrapper.toOptional();\n",
                java_option_class_name(inner)
            ),
            Some(format!("{name}Wrapper")),
        ),
        WrapperType::Enum(enum_name) => (
            format!("var {value_name} = {enum_name}.fromNative({name});\n"),
            None,
        ),
        _ => (format!("var {value_name} = {name};\n"), None),
    }
}

fn trait_result_ok_call(method_name: &str, inner: &WrapperType, value: &str) -> String {
    let handle = format!(
        "RESULT_OK_{}",
        to_java_method_name(method_name).to_uppercase()
    );
    match inner {
        WrapperType::UnitExpr => format!("return (MemorySegment) {handle}.invokeExact();"),
        WrapperType::String => format!(
            "try (var valueArena = Arena.ofConfined()) {{\n    return (MemorySegment) {handle}.invokeExact(valueArena.allocateUtf8String({value}), (long) {value}.getBytes(StandardCharsets.UTF_8).length);\n}}"
        ),
        WrapperType::Vec(vec_inner) => format!(
            "try (var resultVec = {}.fromList({value})) {{\n    return (MemorySegment) {handle}.invokeExact(resultVec.rawPtr());\n}}",
            java_vec_class_name(vec_inner)
        ),
        WrapperType::Option(option_inner) => format!(
            "return (MemorySegment) {handle}.invokeExact({}.fromOptional({value}));",
            java_option_class_name(option_inner)
        ),
        WrapperType::Struct(_) => {
            format!("return (MemorySegment) {handle}.invokeExact({value}.rawPtr());")
        }
        WrapperType::Enum(_) => {
            format!("return (MemorySegment) {handle}.invokeExact({value}.toNative());")
        }
        _ => format!("return (MemorySegment) {handle}.invokeExact({value});"),
    }
}

fn trait_callback_success(function: &crate::wrapper::FunctionWrapper) -> String {
    let name = function.name.to_string();
    let method = to_java_method_name(&name);
    let call_args = function
        .args
        .iter()
        .map(|arg| format!("{}Value", arg.arg_name))
        .collect::<Vec<_>>()
        .join(", ");
    let call = format!("state.target.{method}({call_args})");
    let Some(return_wrapper) = function.return_wrapper.as_ref() else {
        return format!("{call};\nreturn;\n");
    };
    if let WrapperType::Result(inner) = &return_wrapper.wrapper_type {
        let success = if matches!(inner.as_ref(), WrapperType::UnitExpr) {
            format!("{call};\n{}", trait_result_ok_call(&name, inner, ""))
        } else {
            let value = format!("{name}Result");
            format!(
                "var {value} = {call};\n{}",
                trait_result_ok_call(&name, inner, &value)
            )
        };
        return success;
    }
    match &return_wrapper.wrapper_type {
        WrapperType::UnitExpr => format!("{call};\nreturn;\n"),
        WrapperType::String => format!(
            "try (var resultArena = Arena.ofConfined()) {{\n    return (MemorySegment) STRING_FROM_C_PTR.invokeExact(resultArena.allocateUtf8String({call}));\n}}"
        ),
        WrapperType::Struct(_) => format!("return {call}.leak();"),
        WrapperType::Vec(inner) => format!(
            "try (var resultVec = {}.fromList({call})) {{\n    return resultVec.leak();\n}}",
            java_vec_class_name(inner)
        ),
        WrapperType::Option(inner) => format!(
            "return {}.fromOptional({call});",
            java_option_class_name(inner)
        ),
        WrapperType::Enum(_) => format!("return {call}.toNative();"),
        _ => format!("return {call};"),
    }
}

fn trait_callback_failure(function: &crate::wrapper::FunctionWrapper) -> String {
    if let Some(crate::wrapper::FunctionReturnWrapper {
        wrapper_type: WrapperType::Result(inner),
        ..
    }) = &function.return_wrapper
    {
        let handle = format!(
            "RESULT_ERR_{}",
            to_java_method_name(&function.name.to_string()).to_uppercase()
        );
        let _ = inner;
        format!(
            "return createErrorResult({handle}, callbackError.getMessage() == null ? callbackError.toString() : callbackError.getMessage());"
        )
    } else {
        let result = trait_callback_return(function)
            .map(|ret| match ret {
                WrapperType::Bool => "false",
                WrapperType::IntegerNumber(_) | WrapperType::Enum(_) => "0",
                WrapperType::FloatingPointNumber(_) => "0",
                _ => "MemorySegment.NULL",
            })
            .unwrap_or("");
        if result.is_empty() {
            "state.record(callbackError);\nreturn;\n".to_string()
        } else {
            format!("state.record(callbackError);\nreturn {result};\n")
        }
    }
}

fn trait_callback_source(function: &crate::wrapper::FunctionWrapper) -> String {
    let method_name = function.name.to_string();
    let callback_name = format!("callback_{method_name}");
    let return_type = trait_callback_return(function)
        .map(trait_callback_carrier)
        .unwrap_or("void");
    let mut parameters = vec!["MemorySegment context".to_string()];
    parameters.extend(function.args.iter().map(|arg| {
        format!(
            "{} {}",
            trait_callback_carrier(&arg.wrapper_type),
            arg.arg_name
        )
    }));
    let mut conversions = String::new();
    let mut resources = Vec::new();
    for arg in &function.args {
        let (conversion, resource) = trait_callback_arg_conversion(arg);
        conversions.push_str(&conversion);
        if let Some(resource) = resource {
            resources.push(resource);
        }
    }
    let result_body = trait_callback_success(function);
    let failure_body = trait_callback_failure(function);
    let protected_body = if resources.is_empty() {
        format!("{conversions}{result_body}")
    } else {
        format!(
            "{conversions}try ({}) {{\n{}\n}}",
            resources.join("; "),
            prepend_each_line_with_n_tabs(&result_body, 2)
        )
    };
    let protected = format!(
        "try {{\n{}}} catch (Throwable callbackError) {{\n{}\n}}",
        prepend_each_line_with_n_tabs(&protected_body, 2),
        prepend_each_line_with_n_tabs(&failure_body, 2)
    );
    format!(
        "    private static {return_type} {callback_name}({}) {{\n        var state = callbackState(context);\n{}\n    }}\n",
        parameters.join(", "),
        prepend_each_line_with_n_tabs(&protected, 2)
    )
}

fn trait_bridge_source(trait_wrapper: &TraitWrapper) -> String {
    let trait_name = trait_wrapper.name.to_string();
    let bridge_name = format!("{trait_name}Bridge");
    let vtable_fields = trait_wrapper
        .functions
        .iter()
        .map(|function| format!("ValueLayout.ADDRESS.withName(\"{}\").", function.name))
        .collect::<Vec<_>>();
    let vtable_layout = vtable_fields
        .iter()
        .map(|field| field.trim_end_matches('.').to_string())
        .collect::<Vec<_>>()
        .join(",\n            ");

    let callback_fields = trait_wrapper
        .functions
        .iter()
        .enumerate()
        .map(|(index, function)| {
            let method = function.name.to_string();
            let callback = format!("callback_{method}");
            format!(
                "            vtable.set(ValueLayout.ADDRESS, {index}L * ValueLayout.ADDRESS.byteSize(), upcall(\"{callback}\", {method_type}, {descriptor}, arena));\n",
                method_type = trait_callback_method_type(function),
                descriptor = trait_callback_descriptor(function)
            )
        })
        .collect::<String>();
    let callbacks = trait_wrapper
        .functions
        .iter()
        .map(trait_callback_source)
        .collect::<String>();
    let result_fields = trait_wrapper
        .functions
        .iter()
        .filter_map(|function| {
            let Some(WrapperType::Result(_inner)) = function
                .return_wrapper
                .as_ref()
                .map(|ret| &ret.wrapper_type)
            else {
                return None;
            };
            let suffix = to_java_method_name(&function.name.to_string()).to_uppercase();
            Some(format!(
                "    private static final MethodHandle RESULT_OK_{suffix};\n    private static final MethodHandle RESULT_ERR_{suffix};\n"
            ))
        })
        .collect::<String>();
    let result_initialization = trait_wrapper
        .functions
        .iter()
        .filter_map(|function| {
            let Some(WrapperType::Result(inner)) = function
                .return_wrapper
                .as_ref()
                .map(|ret| &ret.wrapper_type)
            else {
                return None;
            };
            let suffix = to_java_method_name(&function.name.to_string()).to_uppercase();
            let symbol_ok = format!("{EXPORTED_SYMBOLS_PREFIX}{}_str_error_result_ok", inner.name());
            let symbol_err = format!("{EXPORTED_SYMBOLS_PREFIX}{}_str_error_result_err", inner.name());
            let ok_descriptor = if matches!(inner.as_ref(), WrapperType::UnitExpr) {
                "FunctionDescriptor.of(ValueLayout.ADDRESS)".to_string()
            } else if matches!(inner.as_ref(), WrapperType::String) {
                "FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG)".to_string()
            } else {
                format!(
                    "FunctionDescriptor.of(ValueLayout.ADDRESS, {})",
                    java_layout(inner)
                )
            };
            Some(format!(
                "        RESULT_OK_{suffix} = LINKER.downcallHandle(LOOKUP.find(\"{symbol_ok}\").orElseThrow(), {ok_descriptor});\n        RESULT_ERR_{suffix} = LINKER.downcallHandle(LOOKUP.find(\"{symbol_err}\").orElseThrow(), FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));\n"
            ))
        })
        .collect::<String>();

    let bridge_drop_name = format!("{trait_name}Bridge_drop");
    format!(
        r#"import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.lang.ref.Cleaner;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicReference;

final class {bridge_name} implements AutoCloseable {{
    private static final Cleaner CLEANER = Cleaner.create();
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LOOKUP;
    private static final MethodHandle STRING_DATA;
    private static final MethodHandle STRING_LENGTH;
    private static final MethodHandle STRING_DROP;
    private static final MethodHandle STRING_FROM_C_PTR;
    private static final MemoryLayout VTABLE_LAYOUT = MemoryLayout.structLayout(
            {vtable_layout});
    private static final MemoryLayout BRIDGE_LAYOUT = MemoryLayout.structLayout(
            ValueLayout.ADDRESS.withName("obj"),
            ValueLayout.ADDRESS.withName("vtable"),
            ValueLayout.ADDRESS.withName("deleter"));
    private static final ConcurrentHashMap<Long, CallbackState> STATES = new ConcurrentHashMap<>();
{result_fields}
    private final MemorySegment bridge;
    private final CallbackState state;

    static {{
        HiFfiLibrary.ensureLoaded();
        LOOKUP = SymbolLookup.loaderLookup();
        STRING_DATA = LINKER.downcallHandle(LOOKUP.find("hiFfi__rust_string_data").orElseThrow(),
                FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        STRING_LENGTH = LINKER.downcallHandle(LOOKUP.find("hiFfi__rust_string_len").orElseThrow(),
                FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        STRING_DROP = LINKER.downcallHandle(LOOKUP.find("hiFfi__rust_string_drop").orElseThrow(),
                FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
        STRING_FROM_C_PTR = LINKER.downcallHandle(LOOKUP.find("hiFfi__rust_string_from_c_ptr").orElseThrow(),
                FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
{result_initialization}    }}

    private {bridge_name}(MemorySegment bridge, CallbackState state) {{
        this.bridge = bridge;
        this.state = state;
    }}

    static MemoryLayout bridgeLayout() {{
        return BRIDGE_LAYOUT;
    }}

    static {bridge_name} of({trait_name} target) {{
        Arena arena = Arena.ofShared();
        var state = new CallbackState(target, arena);
        var context = arena.allocate(1, 1);
        STATES.put(context.address(), state);
        var vtable = arena.allocate(VTABLE_LAYOUT);
{callback_fields}        var bridge = arena.allocate(BRIDGE_LAYOUT);
        bridge.set(ValueLayout.ADDRESS, BRIDGE_LAYOUT.byteOffset(MemoryLayout.PathElement.groupElement("obj")), context);
        bridge.set(ValueLayout.ADDRESS, BRIDGE_LAYOUT.byteOffset(MemoryLayout.PathElement.groupElement("vtable")), vtable);
        bridge.set(ValueLayout.ADDRESS, BRIDGE_LAYOUT.byteOffset(MemoryLayout.PathElement.groupElement("deleter")),
                upcall("{bridge_drop_name}", MethodType.methodType(void.class, MemorySegment.class),
                        FunctionDescriptor.ofVoid(ValueLayout.ADDRESS), arena));
        return new {bridge_name}(bridge, state);
    }}

    MemorySegment rawPtr() {{
        return bridge;
    }}

    private static MemorySegment upcall(String name, MethodType type,
            FunctionDescriptor descriptor, Arena arena) {{
        try {{
            var handle = MethodHandles.lookup().findStatic({bridge_name}.class, name, type);
            return LINKER.upcallStub(handle, descriptor, arena);
        }} catch (Throwable error) {{
            throw new ExceptionInInitializerError(error);
        }}
    }}

    private static CallbackState callbackState(MemorySegment context) {{
        var state = STATES.get(context.address());
        if (state == null) {{
            throw new IllegalStateException("Trait callback state is no longer available");
        }}
        return state;
    }}

    private static void {bridge_drop_name}(MemorySegment context) {{
        STATES.remove(context.address());
    }}

    private static String readString(MemorySegment string) throws Throwable {{
        var data = (MemorySegment) STRING_DATA.invokeExact(string);
        long length = (long) STRING_LENGTH.invokeExact(string);
        try {{
            return new String(data.reinterpret(length).toArray(ValueLayout.JAVA_BYTE), StandardCharsets.UTF_8);
        }} finally {{
            STRING_DROP.invokeExact(string);
        }}
    }}

    private static MemorySegment createErrorResult(MethodHandle handle, String message) {{
        try (var errorArena = Arena.ofConfined()) {{
            return (MemorySegment) handle.invokeExact(errorArena.allocateUtf8String(message));
        }} catch (Throwable error) {{
            throw new RuntimeException("Unable to create trait callback error result", error);
        }}
    }}

    private static final class CallbackState {{
        private final {trait_name} target;
        private final AtomicReference<Throwable> failure = new AtomicReference<>();

        private CallbackState({trait_name} target, Arena arena) {{
            this.target = target;
            CLEANER.register(this, arena::close);
        }}

        private void record(Throwable error) {{
            failure.compareAndSet(null, error);
        }}

    }}

{callbacks}
    @Override
    public void close() {{
        var error = state.failure.get();
        if (error != null) {{
            throw new RuntimeException("Java trait callback failed", error);
        }}
    }}
}}
"#
    )
}

fn trait_object_source(trait_wrapper: &TraitWrapper) -> String {
    let trait_name = trait_wrapper.name.to_string();
    let object_name = format!("{trait_name}Object");
    let imports = trait_imports(&trait_wrapper.functions);
    let mut method_fields = String::new();
    let mut method_initializers = String::new();
    let methods = trait_wrapper
        .functions
        .iter()
        .map(|function| {
            let handle_name = format!("METHOD_{}", function.name.to_string().to_uppercase());
            method_fields.push_str(&format!("    private static final MethodHandle {handle_name};\n"));
            let mut layouts = vec!["ValueLayout.ADDRESS".to_string()];
            layouts.extend(function.args.iter().map(|arg| java_argument_layout(&arg.wrapper_type)));
            let ret = function.return_wrapper.as_ref().map(|ret| java_layout(&ret.wrapper_type));
            let descriptor = java_function_descriptor(&layouts, ret);
            let symbol = format!("{}_BoxDyn", function.extern_function_name);
            method_initializers.push_str(&format!(
                "        {handle_name} = LINKER.downcallHandle(LOOKUP.find(\"{symbol}\").orElseThrow(), {descriptor});\n"
            ));

            let return_type = function
                .return_wrapper
                .as_ref()
                .map(|ret| java_return_type(&ret.wrapper_type))
                .unwrap_or_else(|| Some("void".to_string()))
                .expect("Java trait return type");
            let parameters = function.args.iter().map(|arg| format!(
                "{} {}",
                java_argument_type(&arg.wrapper_type).expect("Java trait argument type"),
                arg.arg_name
            )).collect::<Vec<_>>().join(", ");
            let mut call_args = vec!["self".to_string()];
            call_args.extend(function.args.iter().map(|arg| java_native_argument(&arg.wrapper_type, &arg.arg_name)));
            let invoke = java_invoke_body(
                function.return_wrapper.as_ref().map(|ret| &ret.wrapper_type),
                &return_type,
                &call_args.join(", "),
            )
            .replace("handle.", &format!("{handle_name}."));
            let invoke = wrap_java_call(
                function.args.iter().map(|arg| (&arg.wrapper_type, &arg.arg_name)),
                invoke,
            );
            format!(
                "    @Override\n    public {return_type} {}({parameters}) {{\n        try {{\n{}\n        }} catch (RuntimeException unexpectedError) {{\n            throw unexpectedError;\n        }} catch (Throwable unexpectedError) {{\n            throw new RuntimeException(unexpectedError);\n        }}\n    }}\n",
                to_java_method_name(&function.name.to_string()),
                prepend_each_line_with_n_tabs(&invoke, 3)
            )
        })
        .collect::<String>();

    format!(
        r#"{imports}import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Arena;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;

final class {object_name} implements {trait_name} {{
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LOOKUP;
    private static final MethodHandle DROP;
    private static final MethodHandle STRING_DATA;
    private static final MethodHandle STRING_LENGTH;
    private static final MethodHandle STRING_DROP;
{method_fields}    private MemorySegment self;

    static {{
        HiFfiLibrary.ensureLoaded();
        LOOKUP = SymbolLookup.loaderLookup();
        DROP = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__{trait_name}_BoxDyn_drop").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
        STRING_DATA = LINKER.downcallHandle(LOOKUP.find("hiFfi__rust_string_data").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        STRING_LENGTH = LINKER.downcallHandle(LOOKUP.find("hiFfi__rust_string_len").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        STRING_DROP = LINKER.downcallHandle(LOOKUP.find("hiFfi__rust_string_drop").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
{method_initializers}    }}

    private {object_name}(MemorySegment self) {{
        this.self = self;
    }}

    static {object_name} fromRaw(MemorySegment self) {{
        return new {object_name}(self);
    }}

{methods}
    private static String readString(MemorySegment string) throws Throwable {{
        var data = (MemorySegment) STRING_DATA.invokeExact(string);
        long length = (long) STRING_LENGTH.invokeExact(string);
        try {{
            return new String(data.reinterpret(length).toArray(ValueLayout.JAVA_BYTE), StandardCharsets.UTF_8);
        }} finally {{
            STRING_DROP.invokeExact(string);
        }}
    }}

    @Override
    public void close() {{
        if (self != null) {{
            try {{
                DROP.invokeExact(self);
            }} catch (Throwable error) {{
                throw new RuntimeException(error);
            }}
            self = null;
        }}
    }}
}}
"#
    )
}

fn insert_partial_eq(path: &Path, class_name: &str, extern_fn_name: &str) {
    let source = std::fs::read_to_string(path).expect("Unable to read Java struct");
    if source.contains("public boolean equals(Object other)") {
        return;
    }
    let equals_method = format!(
        r#"    @Override
    public boolean equals(Object other) {{
        if (this == other) {{
            return true;
        }}
        if (!(other instanceof {class_name} otherStruct)) {{
            return false;
        }}
        try {{
            return (boolean) PARTIAL_EQ.invokeExact(self, otherStruct.self);
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}

    @Override
    public int hashCode() {{
        return 1;
    }}

"#
    );
    let mut source = insert_ext_method_handler(source, "PARTIAL_EQ");
    source = insert_linker_initialization(
        source,
        "PARTIAL_EQ",
        extern_fn_name,
        "FunctionDescriptor.of(ValueLayout.JAVA_BOOLEAN, ValueLayout.ADDRESS, ValueLayout.ADDRESS)",
    );
    let marker = "    @Override\n    public void close()";
    source = source.replace(marker, &format!("{equals_method}\n{marker}"));
    create_file(source, path);
}

fn insert_linker_initialization(
    source: String,
    handle_name: &str,
    extern_fn_name: &str,
    descriptor: &str,
) -> String {
    source.replace(
        LINKER_INITIALIZATION_MARKER,
        &format!(
            "{LINKER_INITIALIZATION_MARKER}
        {handle_name} = LINKER.downcallHandle(
            LOOKUP.find(\"{extern_fn_name}\").orElseThrow(),
            {descriptor}
        );\n"
        ),
    )
}

fn insert_ext_method_handler(source: String, handle_name: &str) -> String {
    source.replace(
        EXTERNAL_METHOD_MARKER,
        &format!(
            "{EXTERNAL_METHOD_MARKER}\n    private static final MethodHandle {handle_name};\n"
        ),
    )
}

fn insert_default_ctor(path: &Path, class_name: &str, extern_fn_name: &str) {
    let source = std::fs::read_to_string(path).expect("Unable to read Java struct");
    if source.contains(&format!("public {class_name}()")) {
        return;
    }
    let mut source = insert_ext_method_handler(source, "DEFAULT");
    source = insert_linker_initialization(
        source,
        "DEFAULT",
        extern_fn_name,
        "FunctionDescriptor.of(ValueLayout.ADDRESS)",
    );

    let ctor = format!(
        r#"    public {class_name}() {{
        try {{
            self = (MemorySegment) DEFAULT.invokeExact();
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}

"#
    );
    let marker = "    @Override\n    public void close()";
    source = source.replace(marker, &format!("{ctor}\n{marker}"));
    create_file(source, path);
}

fn impl_methods_source(impl_wrapper: &ImplBlockWrapper) -> String {
    impl_wrapper
        .methods
        .iter()
        .filter(|method| method.public)
        .map(impl_method_source)
        .collect::<String>()
}

fn impl_method_source(method: &crate::wrapper::impl_block_wrapper::MethodWrapper) -> String {
    let Some(argument_types) = method
        .args
        .iter()
        .map(|argument| java_argument_type(&argument.wrapper_type))
        .collect::<Option<Vec<_>>>()
    else {
        return String::new();
    };
    let return_type = method
        .return_wrapper
        .as_ref()
        .map(|return_wrapper| java_return_type(&return_wrapper.wrapper_type))
        .unwrap_or_else(|| Some("void".to_string()));

    let Some(return_type) = return_type else {
        return String::new();
    };

    let parameters = method
        .args
        .iter()
        .zip(&argument_types)
        .map(|(arg, ty)| format!("{ty} {}", arg.arg_name))
        .collect::<Vec<_>>()
        .join(", ");

    let layouts = method
        .args
        .iter()
        .map(|arg| java_argument_layout(&arg.wrapper_type))
        .collect::<Vec<_>>();
    let mut descriptor_layouts = Vec::new();
    if !method.is_static {
        descriptor_layouts.push("ValueLayout.ADDRESS".to_string());
    }
    descriptor_layouts.extend(layouts);
    let return_layout = method
        .return_wrapper
        .as_ref()
        .map(|ret| java_layout(&ret.wrapper_type));
    let descriptor = java_function_descriptor(&descriptor_layouts, return_layout);

    let mut call_args = Vec::new();
    if !method.is_static {
        call_args.push("self".to_string());
    }
    call_args.extend(
        method
            .args
            .iter()
            .map(|arg| java_native_argument(&arg.wrapper_type, &arg.arg_name)),
    );
    let call_args = call_args.join(", ");

    let invoke = java_invoke_body(
        method.return_wrapper.as_ref().map(|ret| &ret.wrapper_type),
        &return_type,
        &call_args,
    );
    let invoke = wrap_java_call(
        method
            .args
            .iter()
            .map(|arg| (&arg.wrapper_type, &arg.arg_name)),
        invoke,
    );
    let static_keyword = if method.is_static { "static " } else { "" };
    let method_name = to_java_method_name(&method.name.to_string());
    java_downcall_method(
        &format!("    public {static_keyword}{return_type} {method_name}({parameters})"),
        &method.extern_function_name,
        &descriptor,
        &invoke,
    )
}

fn java_downcall_method(signature: &str, symbol: &str, descriptor: &str, invoke: &str) -> String {
    format!(
        r#"{signature} {{
        var handle = LINKER.downcallHandle(
            LOOKUP.find("{symbol}").orElseThrow(),
            {descriptor}
        );
        try {{
            {invoke}
        }} catch (RuntimeException unexpectedError) {{
            throw unexpectedError;
        }} catch (Throwable unexpectedError) {{
            throw new RuntimeException(unexpectedError);
        }}
    }}
"#
    )
}

fn java_invoke_body(
    return_wrapper: Option<&WrapperType>,
    return_type: &str,
    call_args: &str,
) -> String {
    match return_wrapper {
        Some(WrapperType::Result(inner)) => {
            let unwrap = if matches!(inner.as_ref(), WrapperType::UnitExpr) {
                "rustResult.unwrap();"
            } else {
                "return rustResult.unwrap();"
            };
            format!(
                "try (var rustResult = {}.fromRaw((MemorySegment) handle.invokeExact({call_args}))) {{
                if (rustResult.isErr()) {{
                    throw new RustException(rustResult.unwrapErr());
                }}
                {unwrap}
            }}",
                java_result_class_name(inner)
            )
        }
        Some(WrapperType::String) => {
            format!("return readString((MemorySegment) handle.invokeExact({call_args}));")
        }
        Some(WrapperType::Struct(name)) => {
            format!("return {name}.fromRaw((MemorySegment) handle.invokeExact({call_args}));")
        }
        Some(WrapperType::Enum(name)) => {
            format!("return {name}.fromNative((int) handle.invokeExact({call_args}));")
        }
        Some(WrapperType::Vec(inner)) => format!(
            "try (var resultVec = {}.fromRaw((MemorySegment) handle.invokeExact({call_args}))) {{
                return resultVec.toList();
            }}",
            java_vec_class_name(inner)
        ),
        Some(WrapperType::Option(inner)) => format!(
            "try (var resultOption = {}.fromRaw((MemorySegment) handle.invokeExact({call_args}))) {{
                return resultOption.toOptional();
            }}",
            java_option_class_name(inner)
        ),
        Some(WrapperType::Trait(name)) => {
            format!("return {name}Object.fromRaw((MemorySegment) handle.invokeExact({call_args}));")
        }
        _ if return_type == "void" => format!("handle.invokeExact({call_args});"),
        _ => format!("return ({return_type}) handle.invokeExact({call_args});"),
    }
}

fn java_native_argument(wrapper_type: &WrapperType, arg_name: &syn::Ident) -> String {
    match wrapper_type {
        WrapperType::String => format!("arena.allocateUtf8String({arg_name})"),
        WrapperType::Struct(_) => format!("{arg_name}.rawPtr()"),
        WrapperType::Vec(_) => format!("{arg_name}_vec.rawPtr()"),
        WrapperType::Option(_inner) => {
            format!("{arg_name}_option.rawPtr()")
        }
        WrapperType::Enum(_) => format!("{arg_name}.toNative()"),
        WrapperType::Trait(_) => format!("{arg_name}_bridge.rawPtr()"),
        _ => arg_name.to_string(),
    }
}

fn java_argument_layout(wrapper_type: &WrapperType) -> String {
    match wrapper_type {
        WrapperType::Trait(name) => format!("{name}Bridge.bridgeLayout()"),
        _ => java_layout(wrapper_type),
    }
}

fn java_function_descriptor(arg_layouts: &[String], return_layout: Option<String>) -> String {
    match return_layout {
        None => format!("FunctionDescriptor.ofVoid({})", arg_layouts.join(", ")),
        Some(layout) if arg_layouts.is_empty() => format!("FunctionDescriptor.of({layout})"),
        Some(layout) => format!(
            "FunctionDescriptor.of({layout}, {})",
            arg_layouts.join(", ")
        ),
    }
}

fn write_enum_code(enum_wrapper: &EnumWrapper) {
    let enum_name = enum_wrapper.name.to_string();
    let mut next_value = 0;
    let mut variants = String::new();
    let mut from_native = String::new();
    for variant in &enum_wrapper.variants {
        let value = variant
            .discriminant
            .as_deref()
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(next_value);
        next_value = value + 1;
        variants.push_str(&format!("    {}({value}),\n", variant.name));
        from_native.push_str(&format!("            case {value} -> {};\n", variant.name));
    }
    variants = variants.trim_end_matches(",\n").to_string() + ";\n";
    let source = format!(
        r#"public enum {enum_name} {{
{variants}
    private final int value;

    {enum_name}(int value) {{
        this.value = value;
    }}

    int toNative() {{
        return value;
    }}

    static {enum_name} fromNative(int value) {{
        return switch (value) {{
{from_native}            default -> throw new IllegalArgumentException("Unknown {enum_name} value: " + value);
        }};
    }}
}}
"#
    );
    create_file(source, java_path().join(format!("{enum_name}.java")));
}

fn java_path() -> std::path::PathBuf {
    let path = Path::new(GEN_CODE_DIR).join(JAVA_CODE_DIR);
    std::fs::create_dir_all(&path).expect("Unable to create java directory");
    path
}

fn write_struct_code(struct_wrapper: &StructWrapper) {
    let class_name = struct_wrapper.name.to_string();
    let has_string_field = struct_wrapper
        .fields
        .iter()
        .any(|field| matches!(field.wrapper_type, WrapperType::String));
    let has_vector_field = struct_wrapper
        .fields
        .iter()
        .any(|field| wrapper_contains_vec(&field.wrapper_type));
    let has_optional_field = struct_wrapper
        .fields
        .iter()
        .any(|field| wrapper_contains_option(&field.wrapper_type));
    let Some(fields) = struct_wrapper
        .fields
        .iter()
        .map(|field| {
            java_argument_type(&field.wrapper_type).map(|field_type| {
                let field_name = field.field_name.to_string();
                let suffix = capitalize(&to_java_method_name(&field_name));

                let getter = field.getter.as_ref().map(|_| {
                    let expression = match &field.wrapper_type {
                        WrapperType::String => format!("return readSlice((MemorySegment) GET_{suffix}.invokeExact(self))"),
                        WrapperType::Struct(name) => format!("return {name}.fromRaw((MemorySegment) GET_{suffix}.invokeExact(self))"),
                        WrapperType::Enum(name) => format!("return {name}.fromNative((int) GET_{suffix}.invokeExact(self))"),
                        WrapperType::Vec(inner) => format!(
                            "var rustVec = {}.fromRaw((MemorySegment) GET_{suffix}.invokeExact(self));\n            var list = rustVec.toList();\n            rustVec.leak();\n            return list",
                            java_vec_class_name(inner)
                        ),
                        WrapperType::Option(inner) => format!(
                            "return {}.toOptional((MemorySegment) GET_{suffix}.invokeExact(self))",
                            java_option_class_name(inner)
                        ),
                        _ => format!("return ({field_type}) GET_{suffix}.invokeExact(self)"),
                    };

                    format!(
                        r#"    public {field_type} get{suffix}() {{
        try {{
            {expression};
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}
"#                    )
                }).unwrap_or_default();

                let setter = field.setter.as_ref().map(|_| {
                    let call = match &field.wrapper_type {
                        WrapperType::String => format!("SET_{suffix}.invokeExact(self, arena.allocateUtf8String(value));"),
                        WrapperType::Struct(_) => format!("SET_{suffix}.invokeExact(self, value.rawPtr());"),
                        WrapperType::Enum(_) => format!("SET_{suffix}.invokeExact(self, value.toNative());"),
                        WrapperType::Vec(inner) => format!(
                            "try (var rustVec = {}.fromList(value)) {{\n                SET_{suffix}.invokeExact(self, rustVec.rawPtr());\n            }}",
                            java_vec_class_name(inner)
                        ),
                        WrapperType::Option(inner) => format!(
                            "try (var rustOption = {}.fromRaw({}.fromOptional(value))) {{\n                SET_{suffix}.invokeExact(self, rustOption.rawPtr());\n            }}",
                            java_option_class_name(inner),
                            java_option_class_name(inner)
                        ),
                        _ => format!("SET_{suffix}.invokeExact(self, value);"),
                    };

                    let body = if matches!(field.wrapper_type, WrapperType::String)
                    {
                        format!(r#"try (var arena = Arena.ofConfined()) {{
                {call}
            }}"#)
                    } else {
                        call
                    };

                    format!(
                        "    public void set{suffix}({field_type} value) {{
        try {{
            {body}
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}
"                    )
                }).unwrap_or_default();

                let handles = format!(
                    "{}{}",
                    field.getter.as_ref().map(|_| format!("    private static final MethodHandle GET_{suffix};\n")).unwrap_or_default(),
                    field.setter.as_ref().map(|_| format!("    private static final MethodHandle SET_{suffix};\n")).unwrap_or_default()
                );

                let java_type_layout = java_layout(&field.wrapper_type);

                let getter_initialization = field.getter.as_ref().map(|_|
                        format!("        GET_{suffix} = LINKER.downcallHandle(
            LOOKUP.find(\"hiFfi__{class_name}__get_{field_name}\").orElseThrow(),
            FunctionDescriptor.of({java_type_layout}, ValueLayout.ADDRESS));\n"
                        )
                    ).unwrap_or_default();

                let setter_initialization = field.setter.as_ref().map(|_|
                        format!("        SET_{suffix} = LINKER.downcallHandle(
            LOOKUP.find(\"hiFfi__{class_name}__set_{field_name}\").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, {java_type_layout}));\n"
                        )
                    ).unwrap_or_default();

                let initialization = format!(
                    "{getter_initialization}{setter_initialization}"
                );

                (handles, initialization, getter, setter)
            })
        })
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    let handles = fields
        .iter()
        .map(|field| field.0.as_str())
        .collect::<String>();

    let initialization = fields
        .iter()
        .map(|field| field.1.as_str())
        .collect::<String>();

    let methods = fields
        .iter()
        .map(|field| format!("{}{}", field.2, field.3))
        .collect::<String>();

    let default_symbol = struct_wrapper
        .default_constructor
        .as_ref()
        .map(|constructor| constructor.extern_fn_name.clone());

    let default_handle = if default_symbol.is_some() {
        "    private static final MethodHandle DEFAULT;
"
    } else {
        ""
    };

    let default_initialization = default_symbol
        .as_ref()
        .map(|symbol| {
            format!(
                r#"        DEFAULT = LINKER.downcallHandle(
            LOOKUP.find("{symbol}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS)
        );
"#
            )
        })
        .unwrap_or_default();

    let public_ctor = default_symbol
        .as_ref()
        .map(|_| {
            format!(
                r#"    public {class_name}() {{
        try {{
            self = (MemorySegment) DEFAULT.invokeExact();
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}

"#
            )
        })
        .unwrap_or_default();

    let partial_eq_handle = struct_wrapper
        .partial_eq
        .as_ref()
        .map(|_| "    private static final MethodHandle PARTIAL_EQ;\n")
        .unwrap_or_default();

    let partial_eq_initialization = struct_wrapper
        .partial_eq
        .as_ref()
        .map(|partial_eq| {
            format!(
                "        PARTIAL_EQ = LINKER.downcallHandle(
            LOOKUP.find(\"{}\").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_BOOLEAN, ValueLayout.ADDRESS, ValueLayout.ADDRESS));\n",
                partial_eq.extern_fn_name
            )
        })
        .unwrap_or_default();

    let partial_eq_methods = struct_wrapper
        .partial_eq
        .as_ref()
        .map(|_| {
            format!(
                "    @Override
    public boolean equals(Object other) {{
        if (this == other) {{
            return true;
        }}
        if (!(other instanceof {class_name} otherStruct)) {{
            return false;
        }}
        try {{
            return (boolean) PARTIAL_EQ.invokeExact(self, otherStruct.self);
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}

    @Override
    public int hashCode() {{
        return 1;
    }}
"
            )
        })
        .unwrap_or_default();

    let drop_symbol = &struct_wrapper.drop_ext_fn_name;

    let arena_import = if has_string_field {
        "import java.lang.foreign.Arena;\n"
    } else {
        Default::default()
    };

    let collection_imports = if has_vector_field {
        "import java.util.List;\n"
    } else {
        Default::default()
    };

    let optional_import = if has_optional_field {
        "import java.util.Optional;\n"
    } else {
        Default::default()
    };

    let string_return_helper = if has_string_field {
        r#"    private static String readString(MemorySegment string) throws Throwable {
        var data = (MemorySegment) STRING_DATA.invokeExact(string);
        long length = (long) STRING_LENGTH.invokeExact(string);
        try {
            return new String(data.reinterpret(length).toArray(ValueLayout.JAVA_BYTE), StandardCharsets.UTF_8);
        } finally {
            STRING_DROP.invokeExact(string);
        }
    }
"#
    } else {
        Default::default()
    };

    let string_handles = if has_string_field {
        {
            "    private static final MethodHandle STRING_DATA;
    private static final MethodHandle STRING_LENGTH;
    private static final MethodHandle STRING_DROP;"
        }
    } else {
        Default::default()
    };

    let string_initialization = if has_string_field {
        {
            "        STRING_DATA = LINKER.downcallHandle(
            LOOKUP.find(\"hiFfi__rust_string_data\").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        STRING_LENGTH = LINKER.downcallHandle(
            LOOKUP.find(\"hiFfi__rust_string_len\").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        STRING_DROP = LINKER.downcallHandle(
            LOOKUP.find(\"hiFfi__rust_string_drop\").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));"
        }
    } else {
        Default::default()
    };

    let source = format!(
        r#"{arena_import}{optional_import}import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
{collection_imports}

public final class {class_name} implements AutoCloseable {{
    // external methods marker
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LOOKUP = SymbolLookup.loaderLookup();
{default_handle}
    private static final MethodHandle DROP;
    private static final MethodHandle SLICE_PTR;
    private static final MethodHandle SLICE_LENGTH;
    private static final MethodHandle SLICE_DROP;
{string_handles}
{partial_eq_handle}
{handles}

    private MemorySegment self;

    static {{
        HiFfiLibrary.ensureLoaded();
        {LINKER_INITIALIZATION_MARKER}
{default_initialization}        DROP = LINKER.downcallHandle(
            LOOKUP.find("{drop_symbol}").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)
        );
        SLICE_PTR = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__slice_ptr").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        SLICE_LENGTH = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__slice_len").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        SLICE_DROP = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__slice_drop").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
{string_initialization}{partial_eq_initialization}{initialization}    }}

{public_ctor}    private {class_name}(MemorySegment self) {{
        this.self = self;
    }}

    static {class_name} fromRaw(MemorySegment self) {{
        return new {class_name}(self);
    }}

    MemorySegment rawPtr() {{
        return self;
    }}

    MemorySegment leak() {{
        var pointer = self;
        self = null;
        return pointer;
    }}

    private static String readSlice(MemorySegment slice) throws Throwable {{
        var data = (MemorySegment) SLICE_PTR.invokeExact(slice);
        long length = (long) SLICE_LENGTH.invokeExact(slice);
        try {{
            return new String(data.reinterpret(length).toArray(ValueLayout.JAVA_BYTE), StandardCharsets.UTF_8);
        }} finally {{
            SLICE_DROP.invokeExact(slice);
        }}
    }}

{string_return_helper}

{methods}
{partial_eq_methods}
    @Override
    public void close() {{
        if (self != null) {{
            try {{
                DROP.invokeExact(self);
            }} catch (Throwable error) {{
                throw new RuntimeException(error);
            }}
            self = null;
        }}
    }}
}}
"#
    );
    create_file(source, java_path().join(format!("{class_name}.java")));
}

fn write_function_code(function_wrapper: &crate::wrapper::FunctionWrapper) {
    let Some(argument_types) = function_wrapper
        .args
        .iter()
        .map(|argument| java_argument_type(&argument.wrapper_type))
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };

    let return_type = function_wrapper
        .return_wrapper
        .as_ref()
        .map(|return_wrapper| java_return_type(&return_wrapper.wrapper_type))
        .unwrap_or_else(|| Some("void".to_string()));

    let Some(return_type) = return_type else {
        return;
    };

    let path = java_path();
    let module_path = path.join("FfiModule.java");

    let has_arena = function_wrapper
        .args
        .iter()
        .any(|argument| matches!(argument.wrapper_type, WrapperType::String));

    let has_collections = function_wrapper
        .args
        .iter()
        .any(|argument| wrapper_contains_vec(&argument.wrapper_type))
        || function_wrapper
            .return_wrapper
            .as_ref()
            .is_some_and(|return_wrapper| wrapper_contains_vec(&return_wrapper.wrapper_type));

    let has_optional = function_wrapper
        .args
        .iter()
        .any(|argument| wrapper_contains_option(&argument.wrapper_type))
        || function_wrapper
            .return_wrapper
            .as_ref()
            .is_some_and(|return_wrapper| wrapper_contains_option(&return_wrapper.wrapper_type));

    JAVA_MODULE_CREATED
        .call_once(|| create_file(java_module_header(has_arena, has_collections), &module_path));

    ensure_java_imports(&module_path, has_arena, has_collections, has_optional);

    let parameter_declaration = function_wrapper
        .args
        .iter()
        .zip(&argument_types)
        .map(|(argument, java_type)| format!("{java_type} {}", argument.arg_name))
        .collect::<Vec<_>>()
        .join(", ");

    let layouts = function_wrapper
        .args
        .iter()
        .map(|argument| java_argument_layout(&argument.wrapper_type))
        .collect::<Vec<_>>();
    let return_layout = function_wrapper
        .return_wrapper
        .as_ref()
        .map(|return_wrapper| java_layout(&return_wrapper.wrapper_type));
    let descriptor = java_function_descriptor(&layouts, return_layout);

    let call_arguments = function_wrapper
        .args
        .iter()
        .map(|argument| java_native_argument(&argument.wrapper_type, &argument.arg_name))
        .collect::<Vec<_>>()
        .join(", ");

    let function_call = java_invoke_body(
        function_wrapper
            .return_wrapper
            .as_ref()
            .map(|return_wrapper| &return_wrapper.wrapper_type),
        &return_type,
        &call_arguments,
    );
    let function_call = wrap_java_call(
        function_wrapper
            .args
            .iter()
            .map(|argument| (&argument.wrapper_type, &argument.arg_name)),
        function_call,
    );
    let method_name = to_java_method_name(&function_wrapper.name.to_string());
    let method = java_downcall_method(
        &format!("\n    public static {return_type} {method_name}({parameter_declaration})"),
        &function_wrapper.extern_function_name,
        &descriptor,
        &function_call,
    );

    let source = std::fs::read_to_string(&module_path).expect("Unable to read Java module");
    create_file(
        source.replace(
            "    // HI_FFI_METHODS\n}",
            &format!("{method}    // HI_FFI_METHODS\n}}"),
        ),
        module_path,
    );
}

fn java_module_header(has_arena: bool, has_collections: bool) -> String {
    let has_arena_import = if has_arena {
        "import java.lang.foreign.Arena;\n"
    } else {
        Default::default()
    };
    let has_collections_import = if has_collections {
        "import java.util.List;\n"
    } else {
        Default::default()
    };

    let imports = format!(
        "{has_arena_import}{has_collections_import}import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
"
    );

    imports
        + r#"public final class FfiModule {
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LOOKUP;
    private static final MethodHandle STRING_DATA;
    private static final MethodHandle STRING_LENGTH;
    private static final MethodHandle STRING_DROP;

    static {
        HiFfiLibrary.ensureLoaded();
        LOOKUP = SymbolLookup.loaderLookup();
        STRING_DATA = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__rust_string_data").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        STRING_LENGTH = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__rust_string_len").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        STRING_DROP = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__rust_string_drop").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
    }

    private FfiModule() {}

    private static String readString(MemorySegment string) throws Throwable {
        var data = (MemorySegment) STRING_DATA.invokeExact(string);
        long length = (long) STRING_LENGTH.invokeExact(string);
        try {
            return new String(data.reinterpret(length).toArray(ValueLayout.JAVA_BYTE), StandardCharsets.UTF_8);
        } finally {
            STRING_DROP.invokeExact(string);
        }
    }

    // HI_FFI_METHODS
}
"#
}

fn ensure_java_imports(
    path: &Path,
    needs_arena: bool,
    needs_collections: bool,
    needs_optional: bool,
) {
    let mut source = std::fs::read_to_string(path).expect("Unable to read Java module");
    let mut imports = String::new();
    if needs_arena && !source.contains("import java.lang.foreign.Arena;") {
        imports.push_str("import java.lang.foreign.Arena;\n");
    }
    if needs_collections && !source.contains("import java.util.List;") {
        imports.push_str("import java.util.List;\n");
    }
    if needs_optional && !source.contains("import java.util.Optional;") {
        imports.push_str("import java.util.Optional;\n");
    }
    if !imports.is_empty() {
        source = format!("{imports}{source}");
        create_file(source, path);
    }
}

fn wrap_java_call<'a>(
    args: impl Iterator<Item = (&'a WrapperType, &'a syn::Ident)>,
    invoke: String,
) -> String {
    let args = args.collect::<Vec<_>>();
    let mut resources = Vec::new();
    if args
        .iter()
        .any(|(wrapper_type, _)| matches!(wrapper_type, WrapperType::String))
    {
        resources.push("var arena = Arena.ofConfined()".to_string());
    }
    for (wrapper_type, arg_name) in args {
        match wrapper_type {
            WrapperType::Vec(inner) => resources.push(format!(
                "var {arg_name}_vec = {}.fromList({arg_name})",
                java_vec_class_name(inner)
            )),
            WrapperType::Option(inner) => resources.push(format!(
                "var {arg_name}_option = {}.fromRaw({}.fromOptional({arg_name}))",
                java_option_class_name(inner),
                java_option_class_name(inner)
            )),
            WrapperType::Trait(name) => resources.push(format!(
                "var {arg_name}_bridge = {name}Bridge.of({arg_name})"
            )),
            _ => {}
        }
    }
    if resources.is_empty() {
        invoke
    } else {
        format!(
            "try ({}) {{
                {invoke}
            }}",
            resources.join("; ")
        )
    }
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

fn to_java_method_name(function_name: &str) -> String {
    let mut parts = function_name.split('_');
    let first = parts.next().unwrap_or_default().to_string();
    first + &parts.map(capitalize).collect::<String>()
}
