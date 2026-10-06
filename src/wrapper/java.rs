use crate::wrapper::WrapperType;
use crate::wrapper::base::*;
use crate::{EXPORTED_SYMBOLS_PREFIX, ReusableWrapper, prepend_each_line_with_n_tabs};

impl ReusableWrapper {
    pub fn java(&self) -> Option<String> {
        match self {
            ReusableWrapper::Vec(inner) => Some(gen_vec_wrapper_java(inner)),
            ReusableWrapper::Result(inner) => Some(gen_result_wrapper_java(inner)),
            ReusableWrapper::Option(inner) => Some(gen_option_wrapper_java(inner)),
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
    let collections_import = if matches!(inner, WrapperType::Vec(_)) {
        "import java.util.List;\n"
    } else {
        ""
    };

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
        _ => format!(
            "({}) UNWRAP.invokeExact(self)",
            java_argument_type(inner).expect("supported option inner type")
        ),
    };

    format!(
        r#"{string_support}import java.lang.foreign.FunctionDescriptor;
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
            "try (var rustVec = {}.fromRaw((MemorySegment) UNWRAP.invokeExact(self))) {{\n    return rustVec.toList();\n}} catch (Throwable error) {{\n    throw new RuntimeException(error);\n}}",
            java_vec_class_name(vec_inner)
        ),
        _ => format!(
            "try {{\n    return ({native_type}) UNWRAP.invokeExact(self);\n}} catch (Throwable error) {{\n    throw new RuntimeException(error);\n}}"
        ),
    }
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
    let push_loop = if matches!(inner, WrapperType::String) {
        format!(
            r#"            try (var arena = Arena.ofConfined()) {{
                for (var value : values) {{
                    PUSH.invokeExact({push_args});
                }}
            }}
"#
        )
    } else {
        format!(
            r#"            for (var value : values) {{
                PUSH.invokeExact({push_args});
            }}
"#
        )
    };

    format!(
        r#"{arena_import}import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
{charset_import}import java.util.ArrayList;
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
