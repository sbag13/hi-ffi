use crate::wrapper::WrapperType;
use crate::wrapper::base::*;
use crate::{EXPORTED_SYMBOLS_PREFIX, ReusableWrapper, prepend_each_line_with_n_tabs};

impl ReusableWrapper {
    pub fn java(&self) -> Option<String> {
        match self {
            ReusableWrapper::Vec(inner) => Some(gen_vec_wrapper_java(inner)),
            ReusableWrapper::Result(inner) => Some(gen_result_wrapper_java(inner)),
            ReusableWrapper::Option(inner) => Some(gen_option_wrapper_java(inner)),
            ReusableWrapper::HashMap(key, value) => Some(gen_hashmap_wrapper_java(key, value)),
        }
    }
}

pub fn java_vec_class_name(inner: &WrapperType) -> String {
    format!("Rust{}Vec", inner.name())
}

pub fn java_result_class_name(inner: &WrapperType) -> String {
    format!("Rust{}Result", inner.name())
}

pub fn java_option_class_name(inner: &WrapperType) -> String {
    format!("Rust{}Option", inner.name())
}

pub fn java_library_loader() -> String {
    let library_name = std::env::var("CARGO_PKG_NAME")
        .expect("Cargo package name expected")
        .replace('-', "_");

    format!(
        r#"import java.nio.file.Path;

final class HiFfiLibrary {{
    private static final String LIBRARY_PROPERTY = "hi-ffi.library";

    static {{
        var library = System.getProperty(LIBRARY_PROPERTY, "{library_name}");
        if (library.isBlank()) {{
            throw new IllegalArgumentException(LIBRARY_PROPERTY + " must not be blank");
        }}
        var libraryPath = Path.of(library);
        if (libraryPath.isAbsolute()) {{
            System.load(libraryPath.toString());
        }} else {{
            System.loadLibrary(library);
        }}
    }}

    private HiFfiLibrary() {{}}

    static void ensureLoaded() {{}}
}}
"#
    )
}

pub fn rust_exception_java() -> String {
    format!(
        r#"import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.lang.ref.Cleaner;
import java.nio.charset.StandardCharsets;
import java.util.Optional;

public class RustException extends RuntimeException {{
    private static final Cleaner CLEANER = Cleaner.create();
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LOOKUP;
    private static final MethodHandle STRING_DATA;
    private static final MethodHandle STRING_LENGTH;
    private static final MethodHandle STRING_DROP;
    private static final MethodHandle ARC_DROP;
    private static final MethodHandle ARC_DESC;
    private static final MethodHandle ARC_SOURCE;
    private static final MethodHandle ARC_CLONE;
    private static final MethodHandle REF_DROP;
    private static final MethodHandle REF_DESC;
    private static final MethodHandle REF_SOURCE;

    private final MemorySegment rustError;
    private final MemorySegment sourceError;
    private final Cleaner.Cleanable cleanable;

    static {{
        HiFfiLibrary.ensureLoaded();
        LOOKUP = SymbolLookup.loaderLookup();
        STRING_DATA = LINKER.downcallHandle(
            LOOKUP.find("{RUST_STRING_DATA_FN_NAME}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        STRING_LENGTH = LINKER.downcallHandle(
            LOOKUP.find("{RUST_STRING_LEN_FN_NAME}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        STRING_DROP = LINKER.downcallHandle(
            LOOKUP.find("{RUST_STRING_DROP_FN_NAME}").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
        ARC_DROP = LINKER.downcallHandle(
            LOOKUP.find("{RUST_ARC_DYN_ERR_DROP_FN_NAME}").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
        ARC_DESC = LINKER.downcallHandle(
            LOOKUP.find("{RUST_ARC_DYN_ERR_DESC_FN_NAME}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        ARC_SOURCE = LINKER.downcallHandle(
            LOOKUP.find("{RUST_ARC_DYN_ERR_SOURCE_FN_NAME}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        ARC_CLONE = LINKER.downcallHandle(
            LOOKUP.find("{RUST_ARC_DYN_ERR_CLONE_FN_NAME}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        REF_DROP = LINKER.downcallHandle(
            LOOKUP.find("{RUST_REF_DYN_ERR_DROP_FN_NAME}").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
        REF_DESC = LINKER.downcallHandle(
            LOOKUP.find("{RUST_REF_DYN_ERR_DESC_FN_NAME}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        REF_SOURCE = LINKER.downcallHandle(
            LOOKUP.find("{RUST_REF_DYN_ERR_SOURCE_FN_NAME}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
    }}

    public RustException(MemorySegment rustError) {{
        this(rustError, MemorySegment.NULL);
    }}

    public RustException(MemorySegment rustError, MemorySegment sourceError) {{
        super(readDescription(rustError, sourceError));
        this.rustError = rustError;
        this.sourceError = sourceError;
        this.cleanable = CLEANER.register(this, new NativeState(rustError, sourceError));
    }}

    public Optional<RustException> source() {{
        try {{
            var sourcePtr = hasSourcePointer()
                ? (MemorySegment) REF_SOURCE.invokeExact(sourceError)
                : (MemorySegment) ARC_SOURCE.invokeExact(rustError);
            if (sourcePtr.address() == 0) {{
                return Optional.empty();
            }}
            var clonedRoot = (MemorySegment) ARC_CLONE.invokeExact(rustError);
            return Optional.of(new RustException(clonedRoot, sourcePtr));
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}

    private boolean hasSourcePointer() {{
        return sourceError != null && sourceError.address() != 0;
    }}

    private static String readDescription(MemorySegment rustError, MemorySegment sourceError) {{
        try {{
            var descriptionPtr = sourceError != null && sourceError.address() != 0
                ? (MemorySegment) REF_DESC.invokeExact(sourceError)
                : (MemorySegment) ARC_DESC.invokeExact(rustError);
            return readString(descriptionPtr);
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
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

    private static final class NativeState implements Runnable {{
        private MemorySegment rustError;
        private MemorySegment sourceError;

        private NativeState(MemorySegment rustError, MemorySegment sourceError) {{
            this.rustError = rustError;
            this.sourceError = sourceError;
        }}

        @Override
        public void run() {{
            try {{
                if (rustError != null && rustError.address() != 0) {{
                    ARC_DROP.invokeExact(rustError);
                    rustError = MemorySegment.NULL;
                }}
                if (sourceError != null && sourceError.address() != 0) {{
                    REF_DROP.invokeExact(sourceError);
                    sourceError = MemorySegment.NULL;
                }}
            }} catch (Throwable error) {{
                throw new RuntimeException(error);
            }}
        }}
    }}
}}
"#
    )
}

fn gen_result_wrapper_java(inner: &WrapperType) -> String {
    let wrapper_name = java_result_class_name(inner);
    let inner_name = inner.name();
    let java_type = java_return_type(inner).expect("supported result inner type");

    let unwrap_signature = if java_type == "void" {
        "void unwrap()".to_string()
    } else {
        format!("{java_type} unwrap()")
    };

    let unwrap_descriptor = match inner {
        WrapperType::UnitExpr => "FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)".to_string(),
        _ => format!(
            "FunctionDescriptor.of({}, ValueLayout.ADDRESS)",
            java_layout(inner)
        ),
    };

    let string_handles = if matches!(inner, WrapperType::String) {
        r#"    private static final MethodHandle STRING_DATA;
    private static final MethodHandle STRING_LENGTH;
    private static final MethodHandle STRING_DROP;
"#
    } else {
        ""
    };

    let string_init = if matches!(inner, WrapperType::String) {
        format!(
            r#"        STRING_DATA = LINKER.downcallHandle(
            LOOKUP.find("{RUST_STRING_DATA_FN_NAME}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        STRING_LENGTH = LINKER.downcallHandle(
            LOOKUP.find("{RUST_STRING_LEN_FN_NAME}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        STRING_DROP = LINKER.downcallHandle(
            LOOKUP.find("{RUST_STRING_DROP_FN_NAME}").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
"#
        )
    } else {
        String::new()
    };

    let string_helper = if matches!(inner, WrapperType::String) {
        r#"
    private static String readString(MemorySegment string) throws Throwable {
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
        ""
    };

    let unwrap_body = java_result_unwrap_body(inner);
    let unwrap_body = prepend_each_line_with_n_tabs(&unwrap_body, 2);

    let charset_import = if matches!(inner, WrapperType::String) {
        "import java.nio.charset.StandardCharsets;\n"
    } else {
        ""
    };
    let collections_import = java_container_imports(inner);

    format!(
        r#"import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
{charset_import}{collections_import}
final class {wrapper_name} implements AutoCloseable {{
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LOOKUP;
    private static final MethodHandle DROP;
    private static final MethodHandle UNWRAP;
    private static final MethodHandle UNWRAP_ERR;
    private static final MethodHandle IS_ERR;
{string_handles}
    private MemorySegment self;

    static {{
        HiFfiLibrary.ensureLoaded();
        LOOKUP = SymbolLookup.loaderLookup();
        DROP = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}drop_{inner_name}_result").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
        UNWRAP = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}unwrap_{inner_name}_result").orElseThrow(),
            {unwrap_descriptor});
        UNWRAP_ERR = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}unwrap_err_{inner_name}_result").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        IS_ERR = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}is_err_{inner_name}_result").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_BOOLEAN, ValueLayout.ADDRESS));
{string_init}    }}

    private {wrapper_name}(MemorySegment self) {{
        this.self = self;
    }}

    static {wrapper_name} fromRaw(MemorySegment self) {{
        return new {wrapper_name}(self);
    }}

    boolean isErr() {{
        try {{
            return (boolean) IS_ERR.invokeExact(self);
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}

    {unwrap_signature} {{
{unwrap_body}
    }}

    MemorySegment unwrapErr() {{
        try {{
            return (MemorySegment) UNWRAP_ERR.invokeExact(self);
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}
{string_helper}
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

fn gen_option_wrapper_java(inner: &WrapperType) -> String {
    let wrapper_name = java_option_class_name(inner);
    let inner_name = inner.name();
    let java_type = java_boxed_type(inner).expect("supported option inner type");
    let some_descriptor = if matches!(inner, WrapperType::String) {
        "FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG)"
            .to_string()
    } else {
        format!(
            "FunctionDescriptor.of(ValueLayout.ADDRESS, {})",
            java_layout(inner)
        )
    };
    let unwrap_descriptor = format!(
        "FunctionDescriptor.of({}, ValueLayout.ADDRESS)",
        java_layout(inner)
    );
    let string_support = if matches!(inner, WrapperType::String) {
        "import java.lang.foreign.Arena;\nimport java.nio.charset.StandardCharsets;\n"
    } else {
        ""
    };
    let container_imports = java_container_imports(inner);
    let string_handles = if matches!(inner, WrapperType::String) {
        "    private static final MethodHandle STRING_DATA;\n    private static final MethodHandle STRING_LENGTH;\n    private static final MethodHandle STRING_DROP;\n"
    } else {
        ""
    };
    let string_init = if matches!(inner, WrapperType::String) {
        r#"        STRING_DATA = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__rust_string_data").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        STRING_LENGTH = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__rust_string_len").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        STRING_DROP = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__rust_string_drop").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
"#
    } else {
        ""
    };
    let from_value = match inner {
        WrapperType::IntegerNumber(ty) => match ty.as_str() {
            "i8" | "u8" => "value.orElseThrow().byteValue()",
            "i16" | "u16" => "value.orElseThrow().shortValue()",
            "i32" | "u32" => "value.orElseThrow().intValue()",
            "i64" | "u64" | "usize" => "value.orElseThrow().longValue()",
            _ => unreachable!(),
        }
        .to_string(),
        WrapperType::FloatingPointNumber(ty) => match ty.as_str() {
            "f32" => "value.orElseThrow().floatValue()",
            "f64" => "value.orElseThrow().doubleValue()",
            _ => unreachable!(),
        }
        .to_string(),
        WrapperType::Bool => "value.orElseThrow().booleanValue()".to_string(),
        WrapperType::String => String::new(),
        WrapperType::Struct(_) => "value.orElseThrow().rawPtr()".to_string(),
        WrapperType::Enum(_) => "value.orElseThrow().toNative()".to_string(),
        WrapperType::Vec(_) | WrapperType::Option(_) | WrapperType::HashMap(_, _) => String::new(),
        _ => unreachable!("unsupported option inner type"),
    };
    let some_call = if matches!(inner, WrapperType::String) {
        r#"try (var arena = Arena.ofConfined()) {
                var text = value.orElseThrow();
                return (MemorySegment) SOME.invokeExact(
                    arena.allocateUtf8String(text),
                    (long) text.getBytes(StandardCharsets.UTF_8).length);
            }"#
        .to_string()
    } else if let WrapperType::Vec(vec_inner) = inner {
        format!(
            "try (var nested = {}.fromList(value.orElseThrow())) {{\n                return (MemorySegment) SOME.invokeExact(nested.rawPtr());\n            }}",
            java_vec_class_name(vec_inner)
        )
    } else if let WrapperType::Option(option_inner) = inner {
        format!(
            "try (var nested = {}.fromRaw({}.fromOptional(value.orElseThrow()))) {{\n                return (MemorySegment) SOME.invokeExact(nested.rawPtr());\n            }}",
            java_option_class_name(option_inner),
            java_option_class_name(option_inner)
        )
    } else if let WrapperType::HashMap(key, map_value) = inner {
        format!(
            "try (var nested = {}.fromMap(value.orElseThrow())) {{\n                return (MemorySegment) SOME.invokeExact(nested.rawPtr());\n            }}",
            java_hashmap_class_name(key, map_value)
        )
    } else {
        format!("return (MemorySegment) SOME.invokeExact({from_value});")
    };
    let read_string = if matches!(inner, WrapperType::String) {
        r#"
    private static String readString(MemorySegment string) throws Throwable {
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
        ""
    };
    let to_value = match inner {
        WrapperType::String => "readString((MemorySegment) UNWRAP.invokeExact(self))".to_string(),
        WrapperType::Struct(name) => {
            format!("{name}.fromRaw((MemorySegment) UNWRAP.invokeExact(self))")
        }
        WrapperType::Enum(name) => {
            format!("{name}.fromNative((int) UNWRAP.invokeExact(self))")
        }
        WrapperType::Vec(vec_inner) => format!(
            "{}.toList((MemorySegment) UNWRAP.invokeExact(self))",
            java_vec_class_name(vec_inner)
        ),
        WrapperType::Option(option_inner) => format!(
            "{}.toOptionalAndClose((MemorySegment) UNWRAP.invokeExact(self))",
            java_option_class_name(option_inner)
        ),
        WrapperType::HashMap(key, value) => format!(
            "{}.toMapAndClose((MemorySegment) UNWRAP.invokeExact(self))",
            java_hashmap_class_name(key, value)
        ),
        _ => format!(
            "({}) UNWRAP.invokeExact(self)",
            java_argument_type(inner).expect("supported option inner type")
        ),
    };

    format!(
        r#"        {string_support}{container_imports}import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.util.Optional;

final class {wrapper_name} implements AutoCloseable {{
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LOOKUP;
    private static final MethodHandle DROP;
    private static final MethodHandle IS_SOME;
    private static final MethodHandle UNWRAP;
    private static final MethodHandle SOME;
    private static final MethodHandle NONE;
{string_handles}
    private MemorySegment self;

    static {{
        HiFfiLibrary.ensureLoaded();
        LOOKUP = SymbolLookup.loaderLookup();
        DROP = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}drop_{inner_name}_option").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
        IS_SOME = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}is_some_{inner_name}_option").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_BOOLEAN, ValueLayout.ADDRESS));
        UNWRAP = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}unwrap_{inner_name}_option").orElseThrow(),
            {unwrap_descriptor});
        SOME = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}some_{inner_name}_option").orElseThrow(),
            {some_descriptor});
        NONE = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}none_{inner_name}_option").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS));
{string_init}    }}

    private {wrapper_name}(MemorySegment self) {{
        this.self = self;
    }}

    static {wrapper_name} fromRaw(MemorySegment self) {{
        return new {wrapper_name}(self);
    }}

    MemorySegment rawPtr() {{
        return self;
    }}

    static MemorySegment fromOptional(Optional<{java_type}> value) {{
        try {{
            if (value.isEmpty()) {{
                return (MemorySegment) NONE.invokeExact();
            }}
            {some_call}
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}

    Optional<{java_type}> toOptional() {{
        return toOptional(self);
    }}

    static Optional<{java_type}> toOptional(MemorySegment self) {{
        try {{
            if (!(boolean) IS_SOME.invokeExact(self)) {{
                return Optional.empty();
            }}
            return Optional.of({to_value});
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}

    static Optional<{java_type}> toOptionalAndClose(MemorySegment self) {{
        try (var wrapper = fromRaw(self)) {{
            return wrapper.toOptional();
        }}
    }}

{read_string}
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

fn java_result_unwrap_body(inner: &WrapperType) -> String {
    let native_type = java_argument_type(inner).unwrap_or_default();
    match inner {
        WrapperType::UnitExpr => "try {\n    UNWRAP.invokeExact(self);\n} catch (Throwable error) {\n    throw new RuntimeException(error);\n}".to_string(),
        WrapperType::String => {
            "try {\n    return readString((MemorySegment) UNWRAP.invokeExact(self));\n} catch (Throwable error) {\n    throw new RuntimeException(error);\n}".to_string()
        }
        WrapperType::Struct(name) => format!(
            "try {{\n    return {name}.fromRaw((MemorySegment) UNWRAP.invokeExact(self));\n}} catch (Throwable error) {{\n    throw new RuntimeException(error);\n}}"
        ),
        WrapperType::Enum(name) => format!(
            "try {{\n    return {name}.fromNative((int) UNWRAP.invokeExact(self));\n}} catch (Throwable error) {{\n    throw new RuntimeException(error);\n}}"
        ),
        WrapperType::Vec(vec_inner) => format!(
            "try {{\n    return {}.toList((MemorySegment) UNWRAP.invokeExact(self));\n}} catch (Throwable error) {{\n    throw new RuntimeException(error);\n}}",
            java_vec_class_name(vec_inner)
        ),
        WrapperType::Option(option_inner) => format!(
            "try {{\n    return {}.toOptionalAndClose((MemorySegment) UNWRAP.invokeExact(self));\n}} catch (Throwable error) {{\n    throw new RuntimeException(error);\n}}",
            java_option_class_name(option_inner)
        ),
        WrapperType::HashMap(key, value) => format!(
            "try (var resultMap = {}.fromRaw((MemorySegment) UNWRAP.invokeExact(self))) {{\n    return resultMap.toMap();\n}} catch (Throwable error) {{\n    throw new RuntimeException(error);\n}}",
            java_hashmap_class_name(key, value)
        ),
        _ => format!(
            "try {{\n    return ({native_type}) UNWRAP.invokeExact(self);\n}} catch (Throwable error) {{\n    throw new RuntimeException(error);\n}}"
        ),
    }
}

fn java_container_imports(wrapper_type: &WrapperType) -> String {
    fn contains_vec(wrapper_type: &WrapperType) -> bool {
        match wrapper_type {
            WrapperType::Vec(_) => true,
            WrapperType::Option(inner) | WrapperType::Result(inner) => contains_vec(inner),
            WrapperType::HashMap(key, value) => contains_vec(key) || contains_vec(value),
            _ => false,
        }
    }

    fn contains_option(wrapper_type: &WrapperType) -> bool {
        match wrapper_type {
            WrapperType::Option(_) => true,
            WrapperType::Vec(inner) | WrapperType::Result(inner) => contains_option(inner),
            WrapperType::HashMap(key, value) => contains_option(key) || contains_option(value),
            _ => false,
        }
    }

    fn contains_hashmap(wrapper_type: &WrapperType) -> bool {
        match wrapper_type {
            WrapperType::HashMap(_, _) => true,
            WrapperType::Option(inner) | WrapperType::Result(inner) | WrapperType::Vec(inner) => {
                contains_hashmap(inner)
            }
            _ => false,
        }
    }

    let mut imports = String::new();
    if contains_vec(wrapper_type) {
        imports.push_str("import java.util.List;\n");
    }
    if contains_option(wrapper_type) {
        imports.push_str("import java.util.Optional;\n");
    }
    if contains_hashmap(wrapper_type) {
        imports.push_str("import java.util.Map;\n");
    }
    imports
}

fn gen_vec_wrapper_java(inner: &WrapperType) -> String {
    let wrapper_name = java_vec_class_name(inner);
    let inner_name = inner.name();
    let element_type = java_boxed_type(inner).expect("supported vector element");
    let native_type = java_argument_type(inner).expect("supported vector element");
    let layout = java_layout(inner);
    let push_descriptor = vector_push_descriptor(inner);
    let arena_import = if matches!(inner, WrapperType::String) {
        "import java.lang.foreign.Arena;\n"
    } else {
        ""
    };
    let charset_import = if matches!(inner, WrapperType::String) {
        "import java.nio.charset.StandardCharsets;\n"
    } else {
        ""
    };
    let container_imports = java_container_imports(inner);
    let string_handles = if matches!(inner, WrapperType::String) {
        r#"    private static final MethodHandle STRING_DATA;
    private static final MethodHandle STRING_LENGTH;
    private static final MethodHandle STRING_DROP;
"#
    } else {
        ""
    };
    let string_init = if matches!(inner, WrapperType::String) {
        r#"        STRING_DATA = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__rust_string_data").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        STRING_LENGTH = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__rust_string_len").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        STRING_DROP = LINKER.downcallHandle(
            LOOKUP.find("hiFfi__rust_string_drop").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
"#
    } else {
        ""
    };
    let string_helper = if matches!(inner, WrapperType::String) {
        r#"
    private static String readString(MemorySegment string) throws Throwable {
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
        ""
    };
    let get_expression = match inner {
        WrapperType::String => {
            "readString((MemorySegment) GET.invokeExact(self, index))".to_string()
        }
        WrapperType::Struct(struct_name) => {
            format!("{struct_name}.fromRaw((MemorySegment) GET.invokeExact(self, index))")
        }
        WrapperType::Enum(enum_name) => {
            format!("{enum_name}.fromNative((int) GET.invokeExact(self, index))")
        }
        WrapperType::Vec(vec_inner) => format!(
            "{}.toList((MemorySegment) GET.invokeExact(self, index))",
            java_vec_class_name(vec_inner)
        ),
        WrapperType::Option(option_inner) => format!(
            "{}.toOptionalAndClose((MemorySegment) GET.invokeExact(self, index))",
            java_option_class_name(option_inner)
        ),
        WrapperType::HashMap(key, value) => format!(
            "{}.toMapAndClose((MemorySegment) GET.invokeExact(self, index))",
            java_hashmap_class_name(key, value)
        ),
        _ => format!("({native_type}) GET.invokeExact(self, index)"),
    };
    let push_value = match inner {
        WrapperType::String => "arena.allocateUtf8String(value)".to_string(),
        WrapperType::Struct(_) => "value.rawPtr()".to_string(),
        WrapperType::Enum(_) => "value.toNative()".to_string(),
        WrapperType::IntegerNumber(inner) => match inner.as_str() {
            "i8" | "u8" => "value.byteValue()",
            "i16" | "u16" => "value.shortValue()",
            "i32" | "u32" => "value.intValue()",
            "i64" | "u64" | "usize" => "value.longValue()",
            _ => unreachable!(),
        }
        .to_string(),
        WrapperType::FloatingPointNumber(inner) => match inner.as_str() {
            "f32" => "value.floatValue()",
            "f64" => "value.doubleValue()",
            _ => unreachable!(),
        }
        .to_string(),
        WrapperType::Bool => "value.booleanValue()".to_string(),
        _ => "value".to_string(),
    };
    let push_args = if matches!(inner, WrapperType::String) {
        format!("self, {push_value}, (long) value.getBytes(StandardCharsets.UTF_8).length")
    } else {
        format!("self, {push_value}")
    };
    let push_loop = match inner {
        WrapperType::String => format!(
            r#"            try (var arena = Arena.ofConfined()) {{
                for (var value : values) {{
                    PUSH.invokeExact({push_args});
                }}
            }}
"#
        ),
        WrapperType::Vec(vec_inner) => format!(
            r#"            for (var value : values) {{
                try (var nested = {}.fromList(value)) {{
                    PUSH.invokeExact(self, nested.rawPtr());
                }}
            }}
"#,
            java_vec_class_name(vec_inner)
        ),
        WrapperType::Option(option_inner) => format!(
            r#"            for (var value : values) {{
                try (var nested = {}.fromRaw({}.fromOptional(value))) {{
                    PUSH.invokeExact(self, nested.rawPtr());
                }}
            }}
"#,
            java_option_class_name(option_inner),
            java_option_class_name(option_inner)
        ),
        WrapperType::HashMap(key, value) => format!(
            r#"            for (var value : values) {{
                try (var nested = {}.fromMap(value)) {{
                    PUSH.invokeExact(self, nested.rawPtr());
                }}
            }}
"#,
            java_hashmap_class_name(key, value)
        ),
        _ => format!(
            r#"            for (var value : values) {{
                PUSH.invokeExact({push_args});
            }}
"#
        ),
    };

    format!(
        r#"{arena_import}import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
{charset_import}{container_imports}import java.util.ArrayList;
import java.util.List;

final class {wrapper_name} implements AutoCloseable {{
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LOOKUP;
    private static final MethodHandle DROP;
    private static final MethodHandle WITH_CAPACITY;
    private static final MethodHandle PUSH;
    private static final MethodHandle LEN;
    private static final MethodHandle GET;
{string_handles}
    private MemorySegment self;

    static {{
        HiFfiLibrary.ensureLoaded();
        LOOKUP = SymbolLookup.loaderLookup();
        DROP = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}drop_{inner_name}_vec").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
        WITH_CAPACITY = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}with_capacity_{inner_name}_vec").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.JAVA_LONG));
        PUSH = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}push_{inner_name}_vec").orElseThrow(),
            {push_descriptor});
        LEN = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}len_{inner_name}_vec").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        GET = LINKER.downcallHandle(
            LOOKUP.find("{EXPORTED_SYMBOLS_PREFIX}get_{inner_name}_vec").orElseThrow(),
            FunctionDescriptor.of({layout}, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG));
{string_init}    }}

    private {wrapper_name}(MemorySegment self) {{
        this.self = self;
    }}

    static {wrapper_name} fromRaw(MemorySegment self) {{
        return new {wrapper_name}(self);
    }}

    static List<{element_type}> toList(MemorySegment self) {{
        try (var wrapper = fromRaw(self)) {{
            return wrapper.toList();
        }}
    }}

    static {wrapper_name} fromList(List<{element_type}> values) {{
        try {{
            var self = (MemorySegment) WITH_CAPACITY.invokeExact((long) values.size());
{push_loop}            return new {wrapper_name}(self);
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}

    MemorySegment rawPtr() {{
        return self;
    }}

    MemorySegment leak() {{
        var pointer = self;
        self = null;
        return pointer;
    }}

    List<{element_type}> toList() {{
        try {{
            long length = (long) LEN.invokeExact(self);
            var result = new ArrayList<{element_type}>();
            for (long index = 0; index < length; index++) {{
                result.add({get_expression});
            }}
            return result;
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}
{string_helper}
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

fn gen_hashmap_wrapper_java(key: &WrapperType, value: &WrapperType) -> String {
    let name = format!("hashmap_{}_{}", key.name(), value.name());
    let class_name = java_hashmap_class_name(key, value);
    let key_type = java_boxed_type(key).expect("supported HashMap key");
    let value_type = java_boxed_type(value).expect("supported HashMap value");
    let new_fn = format!("{EXPORTED_SYMBOLS_PREFIX}new_{name}");
    let drop_fn = format!("{EXPORTED_SYMBOLS_PREFIX}drop_{name}");
    let insert_fn = format!("{EXPORTED_SYMBOLS_PREFIX}insert_{name}");
    let len_fn = format!("{EXPORTED_SYMBOLS_PREFIX}len_{name}");
    let key_at_fn = format!("{EXPORTED_SYMBOLS_PREFIX}key_at_{name}");
    let value_at_fn = format!("{EXPORTED_SYMBOLS_PREFIX}value_at_{name}");
    let key_layout = java_layout(key);
    let value_layout = java_layout(value);
    let key_insert_desc = java_map_insert_layout(key);
    let value_insert_desc = java_map_insert_layout(value);
    let (key_local, key_expr) = java_map_insert_parts(key, "key", "nativeKey");
    let (value_local, value_expr) = java_map_insert_parts(value, "value", "nativeValue");
    let key_read = java_map_read_expression(key, "KEY_AT.invokeExact(self, index)");
    let value_read = java_map_read_expression(value, "VALUE_AT.invokeExact(self, index)");
    let key_string_support = matches!(key, WrapperType::String);
    let value_string_support = matches!(value, WrapperType::String);
    let needs_string = key_string_support || value_string_support;
    let string_imports = if needs_string {
        "import java.nio.charset.StandardCharsets;\n"
    } else {
        ""
    };
    let container_imports = java_container_imports(value)
        .lines()
        .filter(|line| *line != "import java.util.Map;")
        .collect::<Vec<_>>()
        .join("\n");
    let string_handles = if needs_string {
        r#"    private static final MethodHandle STRING_DATA;
    private static final MethodHandle STRING_LENGTH;
    private static final MethodHandle STRING_DROP;
"#
        .to_string()
    } else {
        String::new()
    };
    let string_init = if needs_string {
        format!(
            r#"        STRING_DATA = LINKER.downcallHandle(
            LOOKUP.find("{RUST_STRING_DATA_FN_NAME}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
        STRING_LENGTH = LINKER.downcallHandle(
            LOOKUP.find("{RUST_STRING_LEN_FN_NAME}").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        STRING_DROP = LINKER.downcallHandle(
            LOOKUP.find("{RUST_STRING_DROP_FN_NAME}").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
"#
        )
    } else {
        String::new()
    };
    let read_string_helper = if needs_string {
        r#"
    private static String readString(MemorySegment string) throws Throwable {
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
        ""
    };
    let call_insert = format!("INSERT.invokeExact(self, {key_expr}, {value_expr});");
    let header = if key_string_support || value_string_support {
        "import java.lang.foreign.Arena;\n"
    } else {
        ""
    };
    let mut entry_resources = Vec::new();
    if needs_string {
        entry_resources.push("var arena = Arena.ofConfined()".to_string());
    }
    if let Some(nested_resource) = java_map_nested_resource(value, "value") {
        entry_resources.push(nested_resource);
    }
    let entry_body =
        format!("{key_local}\n                {value_local}\n                {call_insert}");
    let add_entry = if entry_resources.is_empty() {
        entry_body
    } else {
        format!(
            "try ({}) {{\n                {entry_body}\n            }}",
            entry_resources.join("; ")
        )
    };
    let insert_descriptor = format!(
        "FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, {key_insert_desc}, {value_insert_desc})"
    );
    format!(
        r#"{header}{string_imports}{container_imports}
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.util.HashMap;
import java.util.Map;

final class {class_name} implements AutoCloseable {{
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LOOKUP;
    private static final MethodHandle NEW;
    private static final MethodHandle DROP;
    private static final MethodHandle INSERT;
    private static final MethodHandle LEN;
    private static final MethodHandle KEY_AT;
    private static final MethodHandle VALUE_AT;
{string_handles}
    private MemorySegment self;

    static {{
        HiFfiLibrary.ensureLoaded();
        LOOKUP = SymbolLookup.loaderLookup();
        NEW = LINKER.downcallHandle(LOOKUP.find("{new_fn}").orElseThrow(),
                FunctionDescriptor.of(ValueLayout.ADDRESS));
        DROP = LINKER.downcallHandle(LOOKUP.find("{drop_fn}").orElseThrow(),
                FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
        INSERT = LINKER.downcallHandle(LOOKUP.find("{insert_fn}").orElseThrow(),
                {insert_descriptor});
        LEN = LINKER.downcallHandle(LOOKUP.find("{len_fn}").orElseThrow(),
                FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
        KEY_AT = LINKER.downcallHandle(LOOKUP.find("{key_at_fn}").orElseThrow(),
                FunctionDescriptor.of({key_layout}, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG));
        VALUE_AT = LINKER.downcallHandle(LOOKUP.find("{value_at_fn}").orElseThrow(),
                FunctionDescriptor.of({value_layout}, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG));
{string_init}    }}

    private {class_name}(MemorySegment self) {{ this.self = self; }}

    static {class_name} fromRaw(MemorySegment self) {{ return new {class_name}(self); }}
    static Map<{key_type}, {value_type}> toMapAndClose(MemorySegment self) {{
        try (var wrapper = fromRaw(self)) {{
            return wrapper.toMap();
        }}
    }}
    MemorySegment rawPtr() {{ return self; }}
    MemorySegment leak() {{
        var raw = self;
        self = null;
        return raw;
    }}

    static {class_name} fromMap(Map<{key_type}, {value_type}> values) {{
        try {{
            var self = (MemorySegment) NEW.invokeExact();
            var result = new {class_name}(self);
            for (var entry : values.entrySet()) {{
                var key = entry.getKey();
                var value = entry.getValue();
                {add_entry}
            }}
            return result;
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}

    Map<{key_type}, {value_type}> toMap() {{
        try {{
            long size = (long) LEN.invokeExact(self);
            var result = new HashMap<{key_type}, {value_type}>();
            for (long index = 0; index < size; index++) {{
                var key = {key_read};
                var value = {value_read};
                result.put(key, value);
            }}
            return result;
        }} catch (Throwable error) {{
            throw new RuntimeException(error);
        }}
    }}
{read_string_helper}
    @Override
    public void close() {{
        if (self != null) {{
            try {{ DROP.invokeExact(self); }}
            catch (Throwable error) {{ throw new RuntimeException(error); }}
            self = null;
        }}
    }}
}}
"#
    )
}

pub(crate) fn java_hashmap_class_name(key: &WrapperType, value: &WrapperType) -> String {
    format!("RustHashMap{}{}", key.name(), value.name())
}

fn java_map_insert_layout(wrapper_type: &WrapperType) -> String {
    match wrapper_type {
        WrapperType::String => "ValueLayout.ADDRESS, ValueLayout.JAVA_LONG".to_string(),
        _ => java_layout(wrapper_type),
    }
}

fn java_map_insert_parts(wrapper_type: &WrapperType, source: &str, name: &str) -> (String, String) {
    match wrapper_type {
        WrapperType::String => (
            format!(
                "var {name} = arena.allocateUtf8String({source});\n                long {name}Length = (long) {source}.getBytes(StandardCharsets.UTF_8).length;"
            ),
            format!("{name}, {name}Length"),
        ),
        WrapperType::IntegerNumber(ty) => match ty.as_str() {
            "i8" | "u8" => (
                format!("byte {name} = {source}.byteValue();"),
                name.to_string(),
            ),
            "i16" | "u16" => (
                format!("short {name} = {source}.shortValue();"),
                name.to_string(),
            ),
            "i32" | "u32" => (
                format!("int {name} = {source}.intValue();"),
                name.to_string(),
            ),
            _ => (
                format!("long {name} = {source}.longValue();"),
                name.to_string(),
            ),
        },
        WrapperType::FloatingPointNumber(ty) if ty == "f32" => (
            format!("float {name} = {source}.floatValue();"),
            name.to_string(),
        ),
        WrapperType::FloatingPointNumber(_) => (
            format!("double {name} = {source}.doubleValue();"),
            name.to_string(),
        ),
        WrapperType::Bool => (
            format!("boolean {name} = {source}.booleanValue();"),
            name.to_string(),
        ),
        WrapperType::Struct(_) => (String::new(), format!("{source}.rawPtr()")),
        WrapperType::Enum(_) => (
            format!("int {name} = {source}.toNative();"),
            name.to_string(),
        ),
        WrapperType::Vec(_) | WrapperType::Option(_) | WrapperType::HashMap(_, _) => {
            (String::new(), format!("{name}Wrapper.rawPtr()"))
        }
        _ => unreachable!("unsupported HashMap key or value"),
    }
}

fn java_map_nested_resource(wrapper_type: &WrapperType, source: &str) -> Option<String> {
    match wrapper_type {
        WrapperType::Vec(inner) => Some(format!(
            "var nativeValueWrapper = {}.fromList({source})",
            java_vec_class_name(inner)
        )),
        WrapperType::Option(inner) => Some(format!(
            "var nativeValueWrapper = {}.fromRaw({}.fromOptional({source}))",
            java_option_class_name(inner),
            java_option_class_name(inner)
        )),
        WrapperType::HashMap(key, value) => Some(format!(
            "var nativeValueWrapper = {}.fromMap({source})",
            java_hashmap_class_name(key, value)
        )),
        _ => None,
    }
}

fn java_map_read_expression(wrapper_type: &WrapperType, call: &str) -> String {
    match wrapper_type {
        WrapperType::String => format!("readString((MemorySegment) {call})"),
        WrapperType::Bool => format!("(boolean) {call}"),
        WrapperType::IntegerNumber(ty) => {
            format!("({}) {call}", java_argument_type(wrapper_type).expect(ty))
        }
        WrapperType::FloatingPointNumber(_) => {
            format!("({}) {call}", java_argument_type(wrapper_type).unwrap())
        }
        WrapperType::Struct(name) => {
            format!("{name}.fromRaw((MemorySegment) {call})")
        }
        WrapperType::Enum(name) => format!("{name}.fromNative((int) {call})"),
        WrapperType::Vec(inner) => format!(
            "{}.toList((MemorySegment) {call})",
            java_vec_class_name(inner)
        ),
        WrapperType::Option(inner) => format!(
            "{}.toOptionalAndClose((MemorySegment) {call})",
            java_option_class_name(inner)
        ),
        WrapperType::HashMap(key, value) => format!(
            "{}.toMapAndClose((MemorySegment) {call})",
            java_hashmap_class_name(key, value)
        ),
        _ => unreachable!("unsupported HashMap key or value"),
    }
}

pub(crate) fn java_argument_type(wrapper_type: &WrapperType) -> Option<String> {
    match wrapper_type {
        WrapperType::IntegerNumber(inner) => Some(
            match inner.as_str() {
                "i8" | "u8" => "byte",
                "i16" | "u16" => "short",
                "i32" | "u32" => "int",
                "i64" | "u64" | "usize" => "long",
                _ => return None,
            }
            .to_string(),
        ),
        WrapperType::FloatingPointNumber(inner) => Some(
            match inner.as_str() {
                "f32" => "float",
                "f64" => "double",
                _ => return None,
            }
            .to_string(),
        ),
        WrapperType::Bool => Some("boolean".to_string()),
        WrapperType::String => Some("String".to_string()),
        WrapperType::Struct(name) => Some(name.clone()),
        WrapperType::Enum(name) => Some(name.clone()),
        WrapperType::Vec(inner) => Some(format!("List<{}>", java_boxed_type(inner)?)),
        WrapperType::Option(inner) => Some(format!("Optional<{}>", java_boxed_type(inner)?)),
        WrapperType::HashMap(key, value) => Some(format!(
            "Map<{}, {}>",
            java_boxed_type(key)?,
            java_boxed_type(value)?
        )),
        WrapperType::Trait(name) => Some(name.clone()),
        _ => None,
    }
}

pub(crate) fn java_return_type(wrapper_type: &WrapperType) -> Option<String> {
    match wrapper_type {
        WrapperType::Result(inner) => java_return_type(inner),
        WrapperType::String => Some("String".to_string()),
        WrapperType::UnitExpr => Some("void".to_string()),
        other => java_argument_type(other),
    }
}

pub(crate) fn java_boxed_type(wrapper_type: &WrapperType) -> Option<String> {
    match wrapper_type {
        WrapperType::IntegerNumber(inner) => Some(
            match inner.as_str() {
                "i8" | "u8" => "Byte",
                "i16" | "u16" => "Short",
                "i32" | "u32" => "Integer",
                "i64" | "u64" | "usize" => "Long",
                _ => return None,
            }
            .to_string(),
        ),
        WrapperType::FloatingPointNumber(inner) => Some(
            match inner.as_str() {
                "f32" => "Float",
                "f64" => "Double",
                _ => return None,
            }
            .to_string(),
        ),
        WrapperType::Bool => Some("Boolean".to_string()),
        WrapperType::String => Some("String".to_string()),
        WrapperType::Struct(name) => Some(name.clone()),
        WrapperType::Enum(name) => Some(name.clone()),
        WrapperType::Vec(inner) => Some(format!("List<{}>", java_boxed_type(inner)?)),
        WrapperType::Option(inner) => Some(format!("Optional<{}>", java_boxed_type(inner)?)),
        WrapperType::HashMap(key, value) => Some(format!(
            "Map<{}, {}>",
            java_boxed_type(key)?,
            java_boxed_type(value)?
        )),
        _ => None,
    }
}

pub(crate) fn java_layout(wrapper_type: &WrapperType) -> String {
    match wrapper_type {
        WrapperType::IntegerNumber(inner) => match inner.as_str() {
            "i8" | "u8" => "ValueLayout.JAVA_BYTE",
            "i16" | "u16" => "ValueLayout.JAVA_SHORT",
            "i32" | "u32" => "ValueLayout.JAVA_INT",
            "i64" | "u64" | "usize" => "ValueLayout.JAVA_LONG",
            _ => unreachable!(),
        },
        WrapperType::FloatingPointNumber(inner) => match inner.as_str() {
            "f32" => "ValueLayout.JAVA_FLOAT",
            "f64" => "ValueLayout.JAVA_DOUBLE",
            _ => unreachable!(),
        },
        WrapperType::Bool => "ValueLayout.JAVA_BOOLEAN",
        WrapperType::Enum(_) => "ValueLayout.JAVA_INT",
        WrapperType::Trait(_) => "ValueLayout.ADDRESS",
        WrapperType::String
        | WrapperType::Struct(_)
        | WrapperType::Vec(_)
        | WrapperType::Option(_)
        | WrapperType::HashMap(_, _)
        | WrapperType::Result(_) => "ValueLayout.ADDRESS",
        _ => unreachable!(),
    }
    .to_string()
}

fn vector_push_descriptor(inner: &WrapperType) -> String {
    let parameters = if matches!(inner, WrapperType::String) {
        "ValueLayout.ADDRESS, ValueLayout.JAVA_LONG".to_string()
    } else {
        java_layout(inner)
    };
    format!("FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, {parameters})")
}
