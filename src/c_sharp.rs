use std::fs;
use std::path::Path;
use std::sync::{LazyLock, Mutex, Once};

use crate::wrapper::{
    FieldWrapper, FunctionWrapper, ParsedWrapper, ReusableWrapper, StructWrapper, Wrapper,
    WrapperType,
    enum_wrapper::EnumWrapper,
    impl_block_wrapper::{ImplBlockWrapper, MethodWrapper},
    trait_wrapper::TraitWrapper,
};
use crate::{GEN_CODE_DIR, create_file};

const C_SHARP_CODE_DIR: &str = "c_sharp/";
static C_SHARP_OUTPUT_INITIALIZED: Once = Once::new();
static C_SHARP_REUSABLE_WRAPPERS: LazyLock<Mutex<std::collections::HashSet<ReusableWrapper>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

pub(crate) fn write_c_sharp_code(wrapper: &Wrapper) {
    c_sharp_output_dir();
    write_c_sharp_reusable_helpers(wrapper);
    match &wrapper.parsed {
        ParsedWrapper::Function(function) => write_c_sharp_function(function),
        ParsedWrapper::Struct(struct_wrapper) => write_c_sharp_struct(struct_wrapper),
        ParsedWrapper::ImplBlock(impl_block) => write_c_sharp_impl(impl_block),
        ParsedWrapper::Enum(enum_wrapper) => write_c_sharp_enum(enum_wrapper),
        ParsedWrapper::Trait(trait_wrapper) => write_c_sharp_trait(trait_wrapper),
    }
}

fn write_c_sharp_reusable_helpers(wrapper: &Wrapper) {
    let mut reusable = C_SHARP_REUSABLE_WRAPPERS.lock().unwrap();
    reusable.extend(wrapper.reusable_wrappers.iter().cloned());
    let library_name = std::env::var("CARGO_PKG_NAME")
        .expect("Cargo package name expected")
        .replace('-', "_");
    let mut helper_sources = reusable
        .iter()
        .filter_map(|wrapper| c_sharp_reusable_helper_source(&library_name, wrapper))
        .collect::<Vec<_>>();
    helper_sources.sort();
    create_file(
        format!(
            "using System;\nusing System.Collections.Generic;\nusing System.Runtime.InteropServices;\nusing System.Text;\n\ninternal static class CSharpReusableHelpers\n{{\n{}\n}}\n",
            helper_sources.join("\n")
        ),
        c_sharp_output_dir().join("CSharpReusableHelpers.cs"),
    );
}

fn c_sharp_reusable_helper_source(library_name: &str, wrapper: &ReusableWrapper) -> Option<String> {
    let source = match wrapper {
        ReusableWrapper::Vec(inner) => {
            let inner_type = c_sharp_type(inner)?;
            let vec_type = c_sharp_type(&WrapperType::Vec(Box::new(inner.clone())))?;
            let prefix = c_sharp_vec_prefix(&vec_type);
            vec_helper_source(library_name, &prefix, &inner_type, &inner.name())
        }
        ReusableWrapper::HashMap(key, value) => {
            let key_type = c_sharp_type(key)?;
            let value_type = c_sharp_type(value)?;
            let map_type = c_sharp_type(&WrapperType::HashMap(
                Box::new(key.clone()),
                Box::new(value.clone()),
            ))?;
            let prefix = c_sharp_map_prefix(&map_type);
            hashmap_helper_source(
                library_name,
                &prefix,
                &key_type,
                &value_type,
                map_type.map_abi_name.as_deref()?,
            )
        }
        ReusableWrapper::Option(inner) => {
            let inner_type = c_sharp_type(inner)?;
            let option_type = c_sharp_type(&WrapperType::Option(Box::new(inner.clone())))?;
            let prefix = c_sharp_option_prefix(&option_type);
            option_helper_source(
                library_name,
                &prefix,
                &inner_type,
                option_type.option_abi_name.as_deref()?,
            )
        }
        ReusableWrapper::Result(inner) => {
            let inner_type = c_sharp_type(inner)?;
            let result_type = c_sharp_type(&WrapperType::Result(Box::new(inner.clone())))?;
            let prefix = c_sharp_result_prefix(&result_type);
            let mut source = result_helper_source(
                library_name,
                &prefix,
                &inner_type,
                result_type.result_abi_name.as_deref()?,
            );
            if matches!(inner_type.conversion, CSharpConversion::Option)
                && inner_type
                    .option_inner
                    .as_deref()
                    .is_some_and(|inner| matches!(inner.conversion, CSharpConversion::String))
            {
                source.push_str(&result_option_string_reader(library_name, &prefix));
            }
            source
        }
    };
    Some(source.replace("private static", "internal static"))
}

fn c_sharp_unique_prefix(kind: &str, abi_name: &str) -> String {
    let safe_abi_name = abi_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("Reusable{kind}_{safe_abi_name}")
}

fn c_sharp_vec_prefix(ty: &CSharpType) -> String {
    c_sharp_unique_prefix("Vec", ty.vec_abi_name.as_deref().expect("vector ABI name"))
}

fn c_sharp_map_prefix(ty: &CSharpType) -> String {
    c_sharp_unique_prefix("Map", ty.map_abi_name.as_deref().expect("map ABI name"))
}

fn c_sharp_option_prefix(ty: &CSharpType) -> String {
    c_sharp_unique_prefix(
        "Option",
        ty.option_abi_name.as_deref().expect("option ABI name"),
    )
}

fn c_sharp_result_prefix(ty: &CSharpType) -> String {
    c_sharp_unique_prefix(
        "Result",
        ty.result_abi_name.as_deref().expect("result ABI name"),
    )
}

fn write_c_sharp_function(function: &FunctionWrapper) {
    write_c_sharp_callable(
        function.name.to_string(),
        &function.extern_function_name,
        &function.args,
        function.return_wrapper.as_ref(),
        "FfiModule",
        true,
        None,
        None,
        format!("{}.cs", function.name),
    );
}

fn write_c_sharp_impl(impl_block: &ImplBlockWrapper) {
    if let Some(constructor) = &impl_block.default_constructor {
        write_c_sharp_default_constructor(
            &impl_block.struct_name.to_string(),
            &constructor.extern_fn_name,
        );
    }

    for method in impl_block.methods.iter().filter(|method| method.public) {
        write_c_sharp_method(&impl_block.struct_name.to_string(), method);
    }
}

fn write_c_sharp_trait(trait_wrapper: &TraitWrapper) {
    let trait_name = trait_wrapper.name.to_string();
    let library_name = std::env::var("CARGO_PKG_NAME")
        .expect("Cargo package name expected")
        .replace('-', "_");
    let interface_methods = trait_wrapper
        .functions
        .iter()
        .map(c_sharp_trait_interface_method)
        .collect::<Vec<_>>()
        .join("\n");
    create_file(
        format!("public interface {trait_name}\n{{\n{interface_methods}\n}}\n"),
        c_sharp_output_dir().join(format!("{trait_name}.cs")),
    );
    create_file(
        c_sharp_trait_callback_source(&library_name, trait_wrapper),
        c_sharp_output_dir().join(format!("{trait_name}Callback.cs")),
    );
    let object_source = format!(
        r#"using System;
using System.Runtime.InteropServices;

public sealed partial class {object_name} : {trait_name}, IDisposable
{{
    private IntPtr Handle;

    internal {object_name}(IntPtr handle)
    {{
        if (handle == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null trait object pointer.");
        Handle = handle;
    }}

    [DllImport("{library_name}", EntryPoint = "hiFfi__{trait_name}_BoxDyn_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void DropNative(IntPtr handle);

    public void Dispose()
    {{
        IntPtr handle = Handle;
        Handle = IntPtr.Zero;
        if (handle != IntPtr.Zero) DropNative(handle);
        GC.SuppressFinalize(this);
    }}

    ~{object_name}() => Dispose();
}}
"#,
        object_name = format!("{trait_name}Object")
    );
    create_file(
        object_source,
        c_sharp_output_dir().join(format!("{trait_name}Object.cs")),
    );
    for function in &trait_wrapper.functions {
        write_c_sharp_callable(
            function.name.to_string(),
            &format!("{}_BoxDyn", function.extern_function_name),
            &function.args,
            function.return_wrapper.as_ref(),
            &format!("{trait_name}Object"),
            false,
            Some("IntPtr"),
            Some("Handle"),
            format!("{trait_name}Object_{}.cs", function.name),
        );
    }
}

fn c_sharp_trait_interface_method(function: &crate::wrapper::FunctionWrapper) -> String {
    let parameters = function
        .args
        .iter()
        .map(|arg| {
            let ty = c_sharp_type(&arg.wrapper_type).expect("C# trait argument type");
            format!("{} {}", ty.managed, arg.arg_name)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let return_type = function
        .return_wrapper
        .as_ref()
        .map(|ret| {
            let ty = c_sharp_type(&ret.wrapper_type).expect("C# trait return type");
            if matches!(ty.conversion, CSharpConversion::Result) {
                result_managed_type(ty.result_inner.as_deref().expect("result inner type"))
            } else if matches!(ty.conversion, CSharpConversion::Vec) {
                format!(
                    "List<{}>",
                    ty.vec_inner.as_deref().expect("vector inner type").managed
                )
            } else {
                ty.managed
            }
        })
        .unwrap_or_else(|| "void".to_string());
    format!(
        "    {return_type} {}({parameters});",
        to_pascal_case(&function.name.to_string())
    )
}

struct CSharpTraitDecoder {
    helpers: Vec<String>,
    method_source: String,
}

fn c_sharp_trait_argument_decoder(
    library_name: &str,
    prefix: &str,
    ty: &CSharpType,
) -> CSharpTraitDecoder {
    let helpers = c_sharp_trait_value_helpers(library_name, prefix, ty);
    let method_name = format!("Decode{prefix}");
    let method_source = match ty.conversion {
        CSharpConversion::Identity => format!(
            "    private static {} {method_name}({} value) => value;",
            ty.managed, ty.native
        ),
        CSharpConversion::Bool => {
            format!("    private static bool {method_name}(byte value) => value != 0;")
        }
        CSharpConversion::Enum => format!(
            "    private static {} {method_name}(int value) => ({})value;",
            ty.managed, ty.managed
        ),
        CSharpConversion::String => format!(
            "    private static string {method_name}(IntPtr value) => ReadRustString(value);"
        ),
        CSharpConversion::StructArgument => format!(
            "    private static {} {method_name}(IntPtr value) => {}.FromNative(value);",
            ty.managed, ty.managed
        ),
        CSharpConversion::Vec => {
            let helper = c_sharp_vec_prefix(ty);
            format!(
                r#"    private static {} {method_name}(IntPtr value)
    {{
        if (value == IntPtr.Zero) throw new InvalidOperationException("Rust passed a null vector pointer.");
        try {{ return {helper}ToList(value); }}
        finally {{ {helper}Drop(value); }}
    }}"#,
                ty.managed
            )
        }
        CSharpConversion::HashMap => {
            let helper = c_sharp_map_prefix(ty);
            format!(
                r#"    private static {} {method_name}(IntPtr value)
    {{
        if (value == IntPtr.Zero) throw new InvalidOperationException("Rust passed a null HashMap pointer.");
        try {{ return {helper}ToDictionary(value); }}
        finally {{ {helper}Drop(value); }}
    }}"#,
                ty.managed
            )
        }
        CSharpConversion::Option => {
            let inner = ty.option_inner.as_deref().expect("option inner type");
            let helper = c_sharp_option_prefix(ty);
            let decode_inner = c_sharp_trait_decode_inner(inner, &helper, "");
            format!(
                r#"    private static {} {method_name}(IntPtr value)
    {{
        if (value == IntPtr.Zero) throw new InvalidOperationException("Rust passed a null option pointer.");
        try
        {{
            if ({helper}IsSome(value) == 0) return null;
            {decode_inner}
        }}
        finally {{ {helper}Drop(value); }}
    }}"#,
                ty.managed
            )
        }
        _ => unreachable!("unsupported C# trait argument conversion"),
    };
    CSharpTraitDecoder {
        helpers,
        method_source,
    }
}

fn c_sharp_trait_decode_inner(ty: &CSharpType, prefix: &str, _nested_prefix: &str) -> String {
    let unwrap = format!("{prefix}Unwrap");
    match ty.conversion {
        CSharpConversion::Identity => format!("return {unwrap}(value);"),
        CSharpConversion::Bool => format!("return {unwrap}(value) != 0;"),
        CSharpConversion::Enum => {
            format!("return ({}){unwrap}(value);", ty.managed)
        }
        CSharpConversion::String => format!("return ReadRustString({unwrap}(value));"),
        CSharpConversion::StructArgument => {
            format!("return {}.FromNative({unwrap}(value));", ty.managed)
        }
        CSharpConversion::Vec => {
            let helper = c_sharp_vec_prefix(ty);
            format!(
                r#"IntPtr nested = {unwrap}(value);
            if (nested == IntPtr.Zero) throw new InvalidOperationException("Rust passed a null nested vector pointer.");
            try {{ return {helper}ToList(nested); }}
            finally {{ {helper}Drop(nested); }}"#
            )
        }
        CSharpConversion::HashMap => {
            let helper = c_sharp_map_prefix(ty);
            format!(
                r#"IntPtr nested = {unwrap}(value);
            if (nested == IntPtr.Zero) throw new InvalidOperationException("Rust passed a null nested HashMap pointer.");
            try {{ return {helper}ToDictionary(nested); }}
            finally {{ {helper}Drop(nested); }}"#
            )
        }
        _ => unreachable!("unsupported C# nested option conversion"),
    }
}

fn c_sharp_trait_value_helpers(
    _library_name: &str,
    _prefix: &str,
    _ty: &CSharpType,
) -> Vec<String> {
    Vec::new()
}

fn c_sharp_trait_return_helpers(library_name: &str, prefix: &str, ty: &CSharpType) -> Vec<String> {
    let mut helpers = c_sharp_trait_value_helpers(library_name, prefix, ty);
    if matches!(ty.conversion, CSharpConversion::Option) {
        helpers.push(c_sharp_trait_option_builder_helpers(
            prefix,
            ty,
            ty.option_inner.as_deref().expect("trait option inner type"),
        ));
    }
    helpers
}

fn c_sharp_trait_option_builder_helpers(
    prefix: &str,
    option_type: &CSharpType,
    inner: &CSharpType,
) -> String {
    let helper = c_sharp_option_prefix(option_type);
    match inner.conversion {
        CSharpConversion::String => format!(
            r#"    private static IntPtr Build{prefix}StringOption(string? value)
    {{
        if (value is null) return {helper}None();
        IntPtr utf8 = Marshal.StringToCoTaskMemUTF8(value);
        if (utf8 == IntPtr.Zero) throw new OutOfMemoryException();
        try {{ return {helper}Some(utf8); }}
        finally {{ Marshal.FreeCoTaskMem(utf8); }}
    }}"#
        ),
        CSharpConversion::Vec => {
            let nested = c_sharp_vec_prefix(inner);
            format!(
                r#"    private static IntPtr Build{prefix}VectorOption({} value)
    {{
        if (value is null) return {helper}None();
        IntPtr nested = {nested}FromList(value);
        try {{ return {helper}Some(nested); }}
        finally {{ {nested}Drop(nested); }}
    }}"#,
                inner.managed
            )
        }
        CSharpConversion::HashMap => {
            let nested = c_sharp_map_prefix(inner);
            format!(
                r#"    private static IntPtr Build{prefix}MapOption({} value)
    {{
        if (value is null) return {helper}None();
        IntPtr nested = {nested}FromDictionary(value);
        try {{ return {helper}Some(nested); }}
        finally {{ {nested}Drop(nested); }}
    }}"#,
                inner.managed
            )
        }
        _ => String::new(),
    }
}

fn c_sharp_trait_result_builder_helpers(
    result_prefix: &str,
    value_prefix: &str,
    inner: &CSharpType,
) -> String {
    match inner.conversion {
        CSharpConversion::String => format!(
            r#"    private static IntPtr Build{result_prefix}StringResult(string value)
    {{
        IntPtr utf8 = Marshal.StringToCoTaskMemUTF8(value);
        if (utf8 == IntPtr.Zero) throw new OutOfMemoryException();
        try {{ return {result_prefix}ResultOk(utf8, (nuint)System.Text.Encoding.UTF8.GetByteCount(value)); }}
        finally {{ Marshal.FreeCoTaskMem(utf8); }}
    }}"#
        ),
        CSharpConversion::StructArgument => format!(
            r#"    private static IntPtr Build{result_prefix}StructResult({} value)
    {{
        bool added = false;
        value.DangerousAddRef(ref added);
        try {{ return {result_prefix}ResultOk(value.DangerousGetHandle()); }}
        finally {{ if (added) value.DangerousRelease(); }}
    }}"#,
            inner.managed
        ),
        CSharpConversion::Vec => {
            let helper = c_sharp_vec_prefix(inner);
            format!(
                r#"    private static IntPtr Build{result_prefix}VectorResult({} value)
    {{
        IntPtr native = {helper}FromList(value);
        try {{ return {result_prefix}ResultOk(native); }}
        finally {{ {helper}Drop(native); }}
    }}"#,
                inner.managed
            )
        }
        CSharpConversion::HashMap => {
            let helper = c_sharp_map_prefix(inner);
            format!(
                r#"    private static IntPtr Build{result_prefix}MapResult({} value)
    {{
        IntPtr native = {helper}FromDictionary(value);
        try {{ return {result_prefix}ResultOk(native); }}
        finally {{ {helper}Drop(native); }}
    }}"#,
                inner.managed
            )
        }
        CSharpConversion::Option => {
            let helper = c_sharp_option_prefix(inner);
            format!(
                r#"    private static IntPtr Build{result_prefix}OptionResult({} value)
    {{
        IntPtr native = {};
        try {{ return {result_prefix}ResultOk(native); }}
        finally {{ {helper}Drop(native); }}
    }}"#,
                inner.managed,
                c_sharp_trait_encode_value(inner, value_prefix, "value", true)
            )
        }
        _ => String::new(),
    }
}

fn c_sharp_result_callback_import(
    library_name: &str,
    prefix: &str,
    inner: &CSharpType,
    abi_name: &str,
) -> String {
    let ok_parameters = match inner.conversion {
        CSharpConversion::Void => String::new(),
        CSharpConversion::String => "IntPtr value, nuint length".to_string(),
        CSharpConversion::Identity => format!("{} value", inner.native),
        CSharpConversion::Bool => "byte value".to_string(),
        CSharpConversion::Enum => "int value".to_string(),
        CSharpConversion::StructArgument
        | CSharpConversion::Vec
        | CSharpConversion::Option
        | CSharpConversion::HashMap => "IntPtr value".to_string(),
        _ => unreachable!("unsupported C# result callback success type"),
    };
    let ok_symbol = format!("hiFfi__{abi_name}_str_error_result_ok");
    let err_symbol = format!("hiFfi__{abi_name}_str_error_result_err");
    format!(
        r#"    [DllImport("{library_name}", EntryPoint = "{ok_symbol}", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {prefix}ResultOk({ok_parameters});
    [DllImport("{library_name}", EntryPoint = "{err_symbol}", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {prefix}ResultErr(IntPtr error);

    private static IntPtr {prefix}ResultError(string message)
    {{
        IntPtr utf8 = Marshal.StringToCoTaskMemUTF8(message);
        if (utf8 == IntPtr.Zero) throw new OutOfMemoryException();
        try {{ return {prefix}ResultErr(utf8); }}
        finally {{ Marshal.FreeCoTaskMem(utf8); }}
    }}
"#,
        ok_symbol = ok_symbol,
        err_symbol = err_symbol,
    )
}

fn c_sharp_trait_encode_value(
    ty: &CSharpType,
    prefix: &str,
    value: &str,
    result_value: bool,
) -> String {
    match ty.conversion {
        CSharpConversion::Identity => value.to_string(),
        CSharpConversion::Bool => format!("{value} ? (byte)1 : (byte)0"),
        CSharpConversion::Enum => format!("(int){value}"),
        CSharpConversion::String => format!(r#"CreateRustString({value})"#),
        CSharpConversion::StructArgument if result_value => {
            format!("{value}.DangerousGetHandle()")
        }
        CSharpConversion::StructArgument => format!("{value}.TakeHandle()"),
        CSharpConversion::Vec => format!("{}FromList({value})", c_sharp_vec_prefix(ty)),
        CSharpConversion::HashMap => {
            format!("{}FromDictionary({value})", c_sharp_map_prefix(ty))
        }
        CSharpConversion::Option => {
            let inner = ty
                .option_inner
                .as_deref()
                .expect("trait option return inner type");
            let helper = c_sharp_option_prefix(ty);
            match inner.conversion {
                CSharpConversion::Identity => {
                    format!("{value}.HasValue ? {helper}Some({value}.Value) : {helper}None()")
                }
                CSharpConversion::Bool => format!(
                    "{value}.HasValue ? {helper}Some({value}.Value ? (byte)1 : (byte)0) : {helper}None()"
                ),
                CSharpConversion::Enum => {
                    format!("{value}.HasValue ? {helper}Some((int){value}.Value) : {helper}None()")
                }
                CSharpConversion::String => {
                    format!("Build{prefix}StringOption({value})")
                }
                CSharpConversion::StructArgument => format!(
                    "{value} is null ? {helper}None() : {helper}Some({value}.DangerousGetHandle())"
                ),
                CSharpConversion::Vec => format!("Build{prefix}VectorOption({value})"),
                CSharpConversion::HashMap => format!("Build{prefix}MapOption({value})"),
                _ => unreachable!("unsupported C# trait option return type"),
            }
        }
        _ => unreachable!("unsupported C# trait return type"),
    }
}

fn c_sharp_trait_encode_result_ok(inner: &CSharpType, prefix: &str, value: &str) -> String {
    let ok = format!("{prefix}ResultOk");
    match inner.conversion {
        CSharpConversion::Void => format!("{ok}()"),
        CSharpConversion::String => format!(r#"Build{prefix}StringResult({value})"#),
        CSharpConversion::StructArgument => format!("Build{prefix}StructResult({value})"),
        CSharpConversion::Vec => format!("Build{prefix}VectorResult({value})"),
        CSharpConversion::HashMap => format!("Build{prefix}MapResult({value})"),
        CSharpConversion::Option => format!("Build{prefix}OptionResult({value})"),
        CSharpConversion::Bool => format!("{ok}({value} ? (byte)1 : (byte)0)"),
        CSharpConversion::Enum => format!("{ok}((int){value})"),
        CSharpConversion::Identity => format!("{ok}({value})"),
        _ => unreachable!("unsupported C# trait result return type"),
    }
}

fn c_sharp_trait_callback_source(library_name: &str, trait_wrapper: &TraitWrapper) -> String {
    let trait_name = trait_wrapper.name.to_string();
    let bridge_name = format!("{trait_name}Bridge");
    let callback_name = format!("{trait_name}Callback");
    let mut vtable_fields = Vec::new();
    let mut vtable_initializers = Vec::new();
    let mut callback_declarations = Vec::new();
    let mut callback_methods = Vec::new();
    let mut helper_sources = Vec::new();
    let mut result_imports = Vec::new();

    for (method_index, function) in trait_wrapper.functions.iter().enumerate() {
        let method = to_pascal_case(&function.name.to_string());
        let field = function.name.to_string();
        let delegate_name = format!("CallbackDelegate{method_index}");
        let callback_field = format!("CallbackFunction{method_index}");
        let callback_method = format!("Invoke{method_index}");
        let mut native_args = vec!["IntPtr context".to_string()];
        let mut invocation_args = Vec::new();
        let mut callback_arg_decodes = Vec::new();

        for (arg_index, arg) in function.args.iter().enumerate() {
            let ty = c_sharp_type(&arg.wrapper_type).expect("C# trait argument type");
            let native_name = format!("arg{arg_index}");
            let prefix = format!("{trait_name}{method}Arg{arg_index}");
            let native_type = match ty.conversion {
                CSharpConversion::Identity | CSharpConversion::Bool | CSharpConversion::Enum => {
                    ty.native.clone()
                }
                _ => "IntPtr".to_string(),
            };
            native_args.push(format!("{native_type} {native_name}"));
            let decoder = c_sharp_trait_argument_decoder(library_name, &prefix, &ty);
            helper_sources.extend(decoder.helpers);
            callback_arg_decodes.push(decoder.method_source);
            invocation_args.push(format!(
                "Decode{trait_name}{method}Arg{arg_index}({native_name})"
            ));
        }

        let return_type = function
            .return_wrapper
            .as_ref()
            .map(|ret| c_sharp_type(&ret.wrapper_type).expect("C# trait return type"));
        let is_result = return_type
            .as_ref()
            .is_some_and(|ty| matches!(ty.conversion, CSharpConversion::Result));
        let result_inner = return_type
            .as_ref()
            .and_then(|ty| ty.result_inner.as_deref());
        let callback_native_return = return_type
            .as_ref()
            .filter(|ty| !matches!(ty.conversion, CSharpConversion::Void))
            .map(|ty| match ty.conversion {
                CSharpConversion::Identity | CSharpConversion::Bool | CSharpConversion::Enum => {
                    ty.native.clone()
                }
                _ => "IntPtr".to_string(),
            })
            .unwrap_or_else(|| "void".to_string());
        let delegate_args = native_args.join(", ");
        let callback_return = if matches!(
            return_type.as_ref().map(|ty| ty.conversion),
            Some(CSharpConversion::Void) | None
        ) {
            " void".to_string()
        } else {
            format!(" {callback_native_return}")
        };
        callback_declarations.push(format!(
            "    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]\n    private delegate{callback_return} {delegate_name}({delegate_args});\n    private static readonly {delegate_name} {callback_field} = {callback_method};"
        ));
        vtable_fields.push(format!("        public IntPtr {field};"));
        vtable_initializers.push(format!(
            "            {field} = Marshal.GetFunctionPointerForDelegate({callback_field}),"
        ));

        let callback_return_signature = if callback_native_return == "void" {
            "void"
        } else {
            &callback_native_return
        };
        let result_ok = result_inner.map(|inner| {
            let prefix = format!("{trait_name}{method}Result");
            result_imports.push(c_sharp_result_callback_import(
                library_name,
                &prefix,
                inner,
                return_type
                    .as_ref()
                    .and_then(|ty| ty.result_abi_name.as_deref())
                    .expect("result callback ABI name"),
            ));
            prefix
        });
        if let Some(inner) = result_inner {
            let value_prefix = format!("{trait_name}{method}ResultValue");
            helper_sources.extend(c_sharp_trait_return_helpers(
                library_name,
                &value_prefix,
                inner,
            ));
            helper_sources.push(c_sharp_trait_result_builder_helpers(
                result_ok.as_deref().expect("result callback helper prefix"),
                &value_prefix,
                inner,
            ));
        } else if let Some(ty) = return_type.as_ref() {
            helper_sources.extend(c_sharp_trait_return_helpers(
                library_name,
                &format!("{trait_name}{method}Return"),
                ty,
            ));
        }

        let invoke_args = invocation_args.join(", ");
        let method_call = format!("state.Target.{method}({invoke_args})");
        let try_body = if let Some(ty) = return_type.as_ref() {
            if matches!(ty.conversion, CSharpConversion::Void) {
                format!("{method_call};")
            } else if is_result {
                let inner = result_inner.expect("result callback inner type");
                if matches!(inner.conversion, CSharpConversion::Void) {
                    format!(
                        "{method_call};\n            return {}ResultOk();",
                        result_ok.as_deref().expect("result callback helpers")
                    )
                } else {
                    let encoded = c_sharp_trait_encode_result_ok(
                        inner,
                        result_ok.as_deref().expect("result callback helpers"),
                        "value",
                    );
                    format!(
                        "var value = {method_call};\n            try {{ return {encoded}; }}\n            finally {{ DisposeTraitReturnValue(value); }}"
                    )
                }
            } else {
                let encoded = c_sharp_trait_encode_value(
                    ty,
                    &format!("{trait_name}{method}Return"),
                    "value",
                    false,
                );
                format!(
                    "var value = {method_call};\n            try {{ return {encoded}; }}\n            finally {{ DisposeTraitReturnValue(value); }}"
                )
            }
        } else {
            format!("{method_call};")
        };
        let catch_body = if is_result {
            let prefix = result_ok.as_deref().expect("result callback helpers");
            format!("return {prefix}ResultError(error.Message);")
        } else if callback_return_signature == "void" {
            "state.Record(error);".to_string()
        } else {
            format!(
                "state.Record(error);\n            return default({callback_return_signature});"
            )
        };
        let signature = format!(
            "private static {callback_return_signature} {callback_method}({delegate_args})"
        );
        callback_methods.push(format!(
            r#"    {signature}
    {{
        var state = ( {callback_name})GCHandle.FromIntPtr(context).Target!;
        try
        {{
            {try_body}
        }}
        catch (Exception error)
        {{
            {catch_body}
        }}
    }}
"#,
            callback_name = callback_name
        ));
        callback_methods.extend(callback_arg_decodes);
    }

    let imports = r#"using System;
        using System.Collections;
        using System.Collections.Generic;
using System.Runtime.InteropServices;
using static CSharpReusableHelpers;
        using System.Text;
        using System.Threading;
"#
    .to_string();
    let helpers = helper_sources.join("\n");
    let result_imports = result_imports.join("\n");
    let vtable_members = vtable_fields.join("\n");
    let vtable_initializers = vtable_initializers.join("\n");
    let callback_declarations = callback_declarations.join("\n\n");
    let callback_methods = callback_methods.join("\n");
    format!(
        r#"{imports}
[StructLayout(LayoutKind.Sequential)]
internal struct {bridge_name}
{{
    public IntPtr Obj;
    public IntPtr VTable;
    public IntPtr Deleter;
}}

internal sealed class {callback_name}
{{
    [StructLayout(LayoutKind.Sequential)]
    private struct VTable
    {{
{vtable_members}
    }}

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void DropDelegate(IntPtr context);
    private static readonly DropDelegate DropFunction = OnDrop;
    private static readonly IntPtr DropPointer = Marshal.GetFunctionPointerForDelegate(DropFunction);
{callback_declarations}
{result_imports}
{helpers}

    internal readonly {trait_name} Target;
    private Exception? _failure;
    private IntPtr _vtable;

    internal {callback_name}({trait_name} target)
    {{
        Target = target ?? throw new ArgumentNullException(nameof(target));
        var table = new VTable
        {{
{vtable_initializers}
        }};
        _vtable = Marshal.AllocHGlobal(Marshal.SizeOf<VTable>());
        if (_vtable == IntPtr.Zero) throw new OutOfMemoryException();
        Marshal.StructureToPtr(table, _vtable, false);
        var self = GCHandle.Alloc(this);
        Bridge = new {bridge_name}
        {{
            Obj = GCHandle.ToIntPtr(self),
            VTable = _vtable,
            Deleter = DropPointer,
        }};
    }}

    internal {bridge_name} Bridge {{ get; }}

    internal void ThrowIfFailed()
    {{
        if (_failure is not null) throw new InvalidOperationException("A managed RustTrait callback failed.", _failure);
    }}

    private void Record(Exception error) =>
        Interlocked.CompareExchange(ref _failure, error, null);

    private static void DisposeTraitReturnValue(object? value)
    {{
        if (value is IDisposable disposable)
        {{
            disposable.Dispose();
        }}
        else if (value is IDictionary dictionary)
        {{
            foreach (DictionaryEntry entry in dictionary) DisposeTraitReturnValue(entry.Value);
        }}
        else if (value is IEnumerable enumerable && value is not string)
        {{
            foreach (var item in enumerable) DisposeTraitReturnValue(item);
        }}
    }}

    private static void OnDrop(IntPtr context)
    {{
        var handle = GCHandle.FromIntPtr(context);
        var state = ({callback_name})handle.Target!;
        if (state._vtable != IntPtr.Zero)
        {{
            Marshal.FreeHGlobal(state._vtable);
            state._vtable = IntPtr.Zero;
        }}
        handle.Free();
    }}

    private static string ReadRustString(IntPtr value)
    {{
        if (value == IntPtr.Zero) throw new InvalidOperationException("Rust passed a null string pointer.");
        try
        {{
            nuint length = RustStringLength(value);
            if (length > int.MaxValue) throw new OverflowException("Rust string is too large for a .NET string.");
            byte[] bytes = new byte[(int)length];
            if (bytes.Length > 0) Marshal.Copy(RustStringData(value), bytes, 0, bytes.Length);
            return System.Text.Encoding.UTF8.GetString(bytes);
        }}
        finally
        {{
            RustStringDrop(value);
        }}
    }}

    private static IntPtr CreateRustString(string value)
    {{
        if (value is null) throw new ArgumentNullException(nameof(value));
        IntPtr utf8 = Marshal.StringToCoTaskMemUTF8(value);
        if (utf8 == IntPtr.Zero) throw new OutOfMemoryException();
        try {{ return RustStringFromCPtr(utf8); }}
        finally {{ Marshal.FreeCoTaskMem(utf8); }}
    }}

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_data", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr RustStringData(IntPtr value);
    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_len", CallingConvention = CallingConvention.Cdecl)]
    private static extern nuint RustStringLength(IntPtr value);
    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void RustStringDrop(IntPtr value);
    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_from_c_ptr", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr RustStringFromCPtr(IntPtr value);

{callback_methods}
}}
"#
    )
}

fn write_c_sharp_default_constructor(struct_name: &str, extern_function_name: &str) {
    let library_name = std::env::var("CARGO_PKG_NAME")
        .expect("Cargo package name expected")
        .replace('-', "_");
    let native_class = format!("{struct_name}DefaultNative");
    let source = format!(
        r#"        using System;
        using System.Runtime.InteropServices;
        using static CSharpReusableHelpers;
using Microsoft.Win32.SafeHandles;

internal static class {native_class}
{{
    [DllImport("{library_name}", EntryPoint = "{extern_function_name}", CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr Create();
}}

public sealed partial class {struct_name}
{{
    public {struct_name}() : base(IntPtr.Zero, true)
    {{
        SetHandle({native_class}.Create());
        if (IsInvalid)
        {{
            throw new InvalidOperationException("Rust returned a null struct pointer.");
        }}
    }}
}}
"#
    );
    create_file(
        source,
        c_sharp_output_dir().join(format!("{struct_name}_DefaultConstructor.cs")),
    );
}

fn write_c_sharp_method(struct_name: &str, method: &MethodWrapper) {
    let method_name = method.name.to_string();
    write_c_sharp_callable(
        method_name.clone(),
        &method.extern_function_name,
        &method.args,
        method.return_wrapper.as_ref(),
        struct_name,
        method.is_static,
        (!method.is_static).then_some(struct_name),
        (!method.is_static).then_some("this"),
        format!("{struct_name}_{method_name}.cs"),
    );
}

#[allow(clippy::too_many_arguments)]
fn write_c_sharp_callable(
    function_name: String,
    extern_function_name: &str,
    function_args: &[crate::wrapper::FunctionArgWrapper],
    function_return: Option<&crate::wrapper::FunctionReturnWrapper>,
    host_class: &str,
    is_static: bool,
    receiver_type: Option<&str>,
    receiver_expression: Option<&str>,
    output_file: String,
) {
    let Some(arguments) = function_args
        .iter()
        .map(|argument| c_sharp_type(&argument.wrapper_type))
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    let return_type = function_return
        .map(|ret| {
            c_sharp_type(&ret.wrapper_type).map(|mut ty| {
                ty.conversion = match ty.conversion {
                    CSharpConversion::String => CSharpConversion::StringReturn,
                    CSharpConversion::StructArgument => {
                        ty.native = "IntPtr".to_string();
                        CSharpConversion::StructReturn
                    }
                    CSharpConversion::Enum => CSharpConversion::EnumReturn,
                    CSharpConversion::Option => CSharpConversion::OptionReturn,
                    CSharpConversion::Vec => {
                        ty.managed = format!(
                            "List<{}>",
                            ty.vec_inner
                                .as_deref()
                                .expect("vector return inner type")
                                .managed
                        );
                        CSharpConversion::VecReturn
                    }
                    CSharpConversion::HashMap => CSharpConversion::HashMapReturn,
                    CSharpConversion::Result => {
                        ty.managed = result_managed_type(
                            ty.result_inner
                                .as_deref()
                                .expect("result return inner type"),
                        );
                        CSharpConversion::ResultReturn
                    }
                    CSharpConversion::TraitObject => {
                        ty.managed = format!("{}Object", ty.managed);
                        ty.native = "IntPtr".to_string();
                        CSharpConversion::TraitObjectReturn
                    }
                    conversion => conversion,
                };
                ty
            })
        })
        .unwrap_or(Some(CSharpType::new(
            "void",
            "void",
            CSharpConversion::Void,
        )));
    let Some(return_type) = return_type else {
        return;
    };

    let method_name = to_pascal_case(&function_name);
    let library_name = std::env::var("CARGO_PKG_NAME")
        .expect("Cargo package name expected")
        .replace('-', "_");
    let parameters = arguments
        .iter()
        .enumerate()
        .map(|(index, ty)| format!("{} arg{index}", ty.managed))
        .collect::<Vec<_>>()
        .join(", ");
    let native_parameters = receiver_type
        .map(|name| format!("{name} self"))
        .into_iter()
        .chain(
            arguments
                .iter()
                .enumerate()
                .map(|(index, ty)| format!("{} arg{index}", ty.native))
                .collect::<Vec<_>>(),
        )
        .collect::<Vec<_>>()
        .join(", ");
    let string_arguments = arguments
        .iter()
        .enumerate()
        .filter_map(|(index, ty)| {
            matches!(ty.conversion, CSharpConversion::String).then_some(index)
        })
        .collect::<Vec<_>>();
    let option_arguments = arguments
        .iter()
        .enumerate()
        .filter_map(|(index, ty)| {
            matches!(ty.conversion, CSharpConversion::Option).then_some(index)
        })
        .collect::<Vec<_>>();
    let vec_arguments = arguments
        .iter()
        .enumerate()
        .filter_map(|(index, ty)| matches!(ty.conversion, CSharpConversion::Vec).then_some(index))
        .collect::<Vec<_>>();
    let hashmap_arguments = arguments
        .iter()
        .enumerate()
        .filter_map(|(index, ty)| {
            matches!(ty.conversion, CSharpConversion::HashMap).then_some(index)
        })
        .collect::<Vec<_>>();
    let trait_arguments = arguments
        .iter()
        .enumerate()
        .filter_map(|(index, ty)| {
            matches!(ty.conversion, CSharpConversion::TraitObject).then_some(index)
        })
        .collect::<Vec<_>>();
    let option_vec_arguments = option_arguments
        .iter()
        .filter(|index| {
            arguments[**index]
                .option_inner
                .as_deref()
                .is_some_and(|inner| matches!(inner.conversion, CSharpConversion::Vec))
        })
        .copied()
        .collect::<Vec<_>>();
    let option_map_arguments = option_arguments
        .iter()
        .filter(|index| {
            arguments[**index]
                .option_inner
                .as_deref()
                .is_some_and(|inner| matches!(inner.conversion, CSharpConversion::HashMap))
        })
        .copied()
        .collect::<Vec<_>>();
    let invocation_args = arguments
        .iter()
        .enumerate()
        .map(|(index, ty)| match ty.conversion {
            CSharpConversion::Identity => format!("arg{index}"),
            CSharpConversion::Bool => format!("arg{index} ? (byte)1 : (byte)0"),
            CSharpConversion::Enum => format!("(int)arg{index}"),
            CSharpConversion::String => format!("arg{index}Utf8"),
            CSharpConversion::StructArgument => format!("arg{index}"),
            CSharpConversion::Option => format!("arg{index}Option"),
            CSharpConversion::Vec => format!("arg{index}Vec"),
            CSharpConversion::HashMap => format!("arg{index}Map"),
            CSharpConversion::TraitObject => format!("arg{index}Bridge"),
            CSharpConversion::StringReturn
            | CSharpConversion::StructReturn
            | CSharpConversion::EnumReturn
            | CSharpConversion::OptionReturn
            | CSharpConversion::VecReturn
            | CSharpConversion::HashMapReturn
            | CSharpConversion::Result
            | CSharpConversion::ResultReturn
            | CSharpConversion::TraitObjectReturn
            | CSharpConversion::Void => {
                unreachable!("invalid function argument conversion")
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let invocation_args = receiver_type.map_or(invocation_args.clone(), |_| {
        if invocation_args.is_empty() {
            receiver_expression.unwrap_or("this").to_string()
        } else {
            format!(
                "{}, {invocation_args}",
                receiver_expression.unwrap_or("this")
            )
        }
    });
    let invocation = format!("{extern_function_name}({invocation_args})");
    let function_body = match return_type.conversion {
        CSharpConversion::Identity => format!("return {invocation};"),
        CSharpConversion::Bool => format!("return {invocation} != 0;"),
        CSharpConversion::EnumReturn => {
            format!("return ({}){invocation};", return_type.managed)
        }
        CSharpConversion::StringReturn => {
            let data_symbol = format!("{extern_function_name}_string_data");
            let len_symbol = format!("{extern_function_name}_string_len");
            let drop_symbol = format!("{extern_function_name}_string_drop");
            format!(
                r#"IntPtr result = {invocation};
        if (result == IntPtr.Zero)
        {{
            throw new InvalidOperationException("Rust returned a null string pointer.");
        }}
        try
        {{
            nuint byteLength = {len_symbol}(result);
            if (byteLength > int.MaxValue)
            {{
                throw new OverflowException("Rust string is too large for a .NET string.");
            }}
            byte[] bytes = new byte[(int)byteLength];
            if (bytes.Length > 0)
            {{
                Marshal.Copy({data_symbol}(result), bytes, 0, bytes.Length);
            }}
            return Encoding.UTF8.GetString(bytes);
        }}
        finally
        {{
            {drop_symbol}(result);
        }}"#
            )
        }
        CSharpConversion::StructReturn => format!(
            r#"IntPtr result = {invocation};
        if (result == IntPtr.Zero)
        {{
            throw new InvalidOperationException("Rust returned a null struct pointer.");
        }}
        return {managed_return}.FromNative(result);"#,
            managed_return = return_type.managed
        ),
        CSharpConversion::OptionReturn => {
            let inner = return_type
                .option_inner
                .as_deref()
                .expect("option return inner type");
            let helpers = c_sharp_option_prefix(&return_type);
            let unwrap = format!("{helpers}Unwrap");
            let is_some = format!("{helpers}IsSome");
            let drop = format!("{helpers}Drop");
            let decode = match inner.conversion {
                CSharpConversion::Identity => format!("return {unwrap}(result);"),
                CSharpConversion::Bool => format!("return {unwrap}(result) != 0;"),
                CSharpConversion::Enum => {
                    format!("return ({}){unwrap}(result);", inner.managed)
                }
                CSharpConversion::String => {
                    let data_symbol = format!("{extern_function_name}_option_string_data");
                    let len_symbol = format!("{extern_function_name}_option_string_len");
                    let drop_symbol = format!("{extern_function_name}_option_string_drop");
                    format!(
                        r#"IntPtr stringResult = {unwrap}(result);
            if (stringResult == IntPtr.Zero)
            {{
                throw new InvalidOperationException("Rust returned a null string pointer inside an option.");
            }}
            try
            {{
                nuint byteLength = {len_symbol}(stringResult);
                if (byteLength > int.MaxValue)
                {{
                    throw new OverflowException("Rust string is too large for a .NET string.");
                }}
                byte[] bytes = new byte[(int)byteLength];
                if (bytes.Length > 0)
                {{
                    Marshal.Copy({data_symbol}(stringResult), bytes, 0, bytes.Length);
                }}
                return Encoding.UTF8.GetString(bytes);
            }}
            finally
            {{
                {drop_symbol}(stringResult);
            }}"#
                    )
                }
                CSharpConversion::StructArgument => format!(
                    r#"IntPtr structResult = {unwrap}(result);
            if (structResult == IntPtr.Zero)
            {{
                throw new InvalidOperationException("Rust returned a null struct pointer inside an option.");
            }}
            return {managed_return}.FromNative(structResult);"#,
                    managed_return = inner.managed
                ),
                CSharpConversion::Vec => {
                    let vector_helper = c_sharp_vec_prefix(inner);
                    format!(
                        r#"IntPtr vectorResult = {unwrap}(result);
            if (vectorResult == IntPtr.Zero)
            {{
                throw new InvalidOperationException("Rust returned a null vector pointer inside an option.");
            }}
            try
            {{
                return {vector_helper}ToList(vectorResult);
            }}
            finally
            {{
                {vector_helper}Drop(vectorResult);
            }}"#
                    )
                }
                CSharpConversion::HashMap => {
                    let map_helper = c_sharp_map_prefix(inner);
                    format!(
                        r#"IntPtr mapResult = {unwrap}(result);
            if (mapResult == IntPtr.Zero)
            {{
                throw new InvalidOperationException("Rust returned a null HashMap pointer inside an option.");
            }}
            try
            {{
                return {map_helper}ToDictionary(mapResult);
            }}
            finally
            {{
                {map_helper}Drop(mapResult);
            }}"#
                    )
                }
                _ => unreachable!("unsupported C# option return inner conversion"),
            };
            format!(
                r#"IntPtr result = {invocation};
        if (result == IntPtr.Zero)
        {{
            throw new InvalidOperationException("Rust returned a null option pointer.");
        }}
        try
        {{
            if ({is_some}(result) == 0) return null;
            {decode}
        }}
        finally
        {{
            {drop}(result);
        }}"#
            )
        }
        CSharpConversion::VecReturn => {
            let helper = c_sharp_vec_prefix(&return_type);
            format!(
                r#"IntPtr result = {invocation};
        if (result == IntPtr.Zero)
        {{
            throw new InvalidOperationException("Rust returned a null vector pointer.");
        }}
        try
        {{
            return {helper}ToList(result);
        }}
        finally
        {{
            {helper}Drop(result);
        }}"#
            )
        }
        CSharpConversion::HashMapReturn => {
            let helper = c_sharp_map_prefix(&return_type);
            format!(
                r#"IntPtr result = {invocation};
        if (result == IntPtr.Zero)
        {{
            throw new InvalidOperationException("Rust returned a null HashMap pointer.");
        }}
        try
        {{
            return {helper}ToDictionary(result);
        }}
        finally
        {{
            {helper}Drop(result);
        }}"#
            )
        }
        CSharpConversion::ResultReturn => {
            let inner = return_type
                .result_inner
                .as_deref()
                .expect("result return inner type");
            let helper = c_sharp_result_prefix(&return_type);
            let err = format!("{helper}UnwrapErr");
            let is_err = format!("{helper}IsErr");
            let drop = format!("{helper}Drop");
            let create_exception = format!("{helper}CreateException");
            let success = match inner.conversion {
                CSharpConversion::Identity => {
                    format!("return {helper}Unwrap(result);")
                }
                CSharpConversion::Bool => {
                    format!("return {helper}Unwrap(result) != 0;")
                }
                CSharpConversion::Enum => format!(
                    "return ({managed_type}){helper}Unwrap(result);",
                    managed_type = inner.managed
                ),
                CSharpConversion::Void => format!("{helper}Unwrap(result);"),
                CSharpConversion::String => {
                    let value = format!("{helper}ReadString");
                    format!("return {value}({helper}Unwrap(result));")
                }
                CSharpConversion::StructArgument => format!(
                    "return {managed_type}.FromNative({helper}Unwrap(result));",
                    managed_type = inner.managed
                ),
                CSharpConversion::Vec => {
                    let vector_helper = c_sharp_vec_prefix(inner);
                    format!(
                        r#"IntPtr value = {helper}Unwrap(result);
            if (value == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null vector result.");
            try
            {{
        return {vector_helper}ToList(value);
            }}
            finally
            {{
        {vector_helper}Drop(value);
            }}"#
                    )
                }
                CSharpConversion::Option => {
                    let option_helper = c_sharp_option_prefix(inner);
                    let option_inner = inner
                        .option_inner
                        .as_deref()
                        .expect("result option value type");
                    let nested_value = match option_inner.conversion {
                        CSharpConversion::Identity => {
                            format!("return {option_helper}Unwrap(option);")
                        }
                        CSharpConversion::Bool => {
                            format!("return {option_helper}Unwrap(option) != 0;")
                        }
                        CSharpConversion::Enum => format!(
                            "return ({}){option_helper}Unwrap(option);",
                            option_inner.managed
                        ),
                        CSharpConversion::String => {
                            format!("return ReadResultOptionString({option_helper}Unwrap(option));")
                        }
                        CSharpConversion::StructArgument => format!(
                            "return {}.FromNative({option_helper}Unwrap(option));",
                            option_inner.managed
                        ),
                        CSharpConversion::Vec => {
                            let vector_helper = c_sharp_vec_prefix(option_inner);
                            format!(
                                r#"IntPtr value = {option_helper}Unwrap(option);
                    if (value == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null vector inside an option result.");
                    try {{ return {vector_helper}ToList(value); }}
                    finally {{ {vector_helper}Drop(value); }}"#
                            )
                        }
                        CSharpConversion::HashMap => {
                            let map_helper = c_sharp_map_prefix(option_inner);
                            format!(
                                r#"IntPtr value = {option_helper}Unwrap(option);
                    if (value == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null HashMap inside an option result.");
                    try {{ return {map_helper}ToDictionary(value); }}
                    finally {{ {map_helper}Drop(value); }}"#
                            )
                        }
                        _ => unreachable!("unsupported C# result option value"),
                    };
                    format!(
                        r#"IntPtr option = {helper}Unwrap(result);
                if (option == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null option result.");
                try
                {{
                    if ({option_helper}IsSome(option) == 0) return null;
                    {nested_value}
                }}
                finally
                {{
                    {option_helper}Drop(option);
                }}"#
                    )
                }
                CSharpConversion::HashMap => {
                    let map_helper = c_sharp_map_prefix(inner);
                    format!(
                        r#"IntPtr value = {helper}Unwrap(result);
            if (value == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null HashMap result.");
            try
            {{
                return {map_helper}ToDictionary(value);
            }}
            finally
            {{
                {map_helper}Drop(value);
            }}"#
                    )
                }
                _ => unreachable!("unsupported C# result inner conversion"),
            };
            let return_success = if matches!(inner.conversion, CSharpConversion::Void) {
                format!("{success}\n            return;")
            } else {
                success
            };
            format!(
                r#"IntPtr result = {invocation};
        if (result == IntPtr.Zero)
        {{
            throw new InvalidOperationException("Rust returned a null result pointer.");
        }}
        try
        {{
            if ({is_err}(result) != 0)
            {{
        IntPtr error = {err}(result);
        if (error == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null error pointer.");
        try
        {{
            throw {create_exception}(error);
        }}
        finally
        {{
            {helper}ArcDrop(error);
        }}
            }}
            {return_success}
        }}
        finally
        {{
            {drop}(result);
        }}"#
            )
        }
        CSharpConversion::TraitObjectReturn => format!(
            r#"IntPtr result = {invocation};
        if (result == IntPtr.Zero)
        {{
            throw new InvalidOperationException("Rust returned a null trait object pointer.");
        }}
        return new {managed_return}(result);"#,
            managed_return = return_type.managed
        ),
        CSharpConversion::Void => format!("{invocation};"),
        CSharpConversion::String
        | CSharpConversion::StructArgument
        | CSharpConversion::Enum
        | CSharpConversion::Option
        | CSharpConversion::Vec
        | CSharpConversion::HashMap
        | CSharpConversion::Result
        | CSharpConversion::TraitObject => {
            unreachable!("invalid function return conversion")
        }
    };
    let body = if string_arguments.is_empty()
        && option_arguments.is_empty()
        && vec_arguments.is_empty()
        && hashmap_arguments.is_empty()
        && trait_arguments.is_empty()
    {
        function_body
    } else {
        let string_declarations = string_arguments
            .iter()
            .map(|index| format!("IntPtr arg{index}Utf8 = IntPtr.Zero;"))
            .collect::<Vec<_>>();
        let option_declarations = option_arguments
            .iter()
            .map(|index| format!("IntPtr arg{index}Option = IntPtr.Zero;"))
            .collect::<Vec<_>>();
        let option_vec_declarations = option_vec_arguments
            .iter()
            .map(|index| format!("IntPtr arg{index}OptionVec = IntPtr.Zero;"))
            .collect::<Vec<_>>();
        let vec_declarations = vec_arguments
            .iter()
            .map(|index| format!("IntPtr arg{index}Vec = IntPtr.Zero;"))
            .collect::<Vec<_>>();
        let hashmap_declarations = hashmap_arguments
            .iter()
            .map(|index| format!("IntPtr arg{index}Map = IntPtr.Zero;"))
            .collect::<Vec<_>>();
        let trait_declarations = trait_arguments
            .iter()
            .map(|index| {
                format!(
                    "{}Callback? arg{index}Callback = null;\n        {}Bridge arg{index}Bridge = default;",
                    arguments[*index].managed,
                    arguments[*index].managed
                )
            })
            .collect::<Vec<_>>();
        let declarations = string_declarations
            .into_iter()
            .chain(option_declarations)
            .chain(option_vec_declarations)
            .chain(vec_declarations)
            .chain(hashmap_declarations)
            .chain(trait_declarations)
            .collect::<Vec<_>>()
            .join("\n        ");
        let string_allocations = string_arguments
            .iter()
            .map(|index| {
                format!(
                    r#"if (arg{index} is null) throw new ArgumentNullException(nameof(arg{index}));
            if (arg{index}.IndexOf('\0') >= 0) throw new ArgumentException("String arguments cannot contain NUL characters.", nameof(arg{index}));
            arg{index}Utf8 = Marshal.StringToCoTaskMemUTF8(arg{index});
            if (arg{index}Utf8 == IntPtr.Zero) throw new OutOfMemoryException();"#
                )
            })
            .collect::<Vec<_>>();
        let option_allocations = option_arguments
            .iter()
            .map(|index| {
                let ty = &arguments[*index];
                let inner = ty.option_inner.as_deref().expect("option inner type");
                let helper = c_sharp_option_prefix(ty);
                match inner.conversion {
                    CSharpConversion::Identity => format!(
                        "arg{index}Option = arg{index}.HasValue ? {helper}Some(arg{index}.Value) : {helper}None();"
                    ),
                    CSharpConversion::Bool => format!(
                        "arg{index}Option = arg{index}.HasValue ? {helper}Some(arg{index}.Value ? (byte)1 : (byte)0) : {helper}None();"
                    ),
                    CSharpConversion::Enum => format!(
                        "arg{index}Option = arg{index}.HasValue ? {helper}Some((int)arg{index}.Value) : {helper}None();"
                    ),
                    CSharpConversion::String => format!(
                        r#"if (arg{index} is null)
            {{
                arg{index}Option = {helper}None();
            }}
            else
            {{
                if (arg{index}.IndexOf('\0') >= 0) throw new ArgumentException("String arguments cannot contain NUL characters.", nameof(arg{index}));
                IntPtr arg{index}OptionUtf8 = Marshal.StringToCoTaskMemUTF8(arg{index});
                if (arg{index}OptionUtf8 == IntPtr.Zero) throw new OutOfMemoryException();
                try
                {{
                    arg{index}Option = {helper}Some(arg{index}OptionUtf8);
                }}
                finally
                {{
                    Marshal.FreeCoTaskMem(arg{index}OptionUtf8);
                }}
            }}"#
                    ),
                                CSharpConversion::StructArgument => format!(
                                    r#"if (arg{index} is null)
                        {{
                            arg{index}Option = {helper}None();
                        }}
                        else
                        {{
                            bool arg{index}AddedReference = false;
                            arg{index}.DangerousAddRef(ref arg{index}AddedReference);
                            try
                            {{
                                arg{index}Option = {helper}Some(arg{index}.DangerousGetHandle());
                            }}
                            finally
                            {{
                                if (arg{index}AddedReference) arg{index}.DangerousRelease();
                            }}
                        }}"#
                                ),
                                CSharpConversion::Vec => {
                                    let vec_helper =
                                        c_sharp_vec_prefix(inner);
                                    let vec_drop = format!("{vec_helper}Drop");
                                    let vec_from_list = format!("{vec_helper}FromList");
                                    format!(
                                        r#"if (arg{index} is null)
                        {{
                            arg{index}Option = {helper}None();
                        }}
                        else
                        {{
                            arg{index}OptionVec = {vec_from_list}(arg{index});
                            try
                            {{
                                arg{index}Option = {helper}Some(arg{index}OptionVec);
                            }}
                            finally
                            {{
                                {vec_drop}(arg{index}OptionVec);
                                arg{index}OptionVec = IntPtr.Zero;
                            }}
                        }}"#
                                    )
                                }
                                CSharpConversion::HashMap => {
                                    let map_helper =
                                        c_sharp_map_prefix(inner);
                                    let map_from = format!("{map_helper}FromDictionary");
                                    let map_drop = format!("{map_helper}Drop");
                                    format!(
                                        r#"if (arg{index} is null)
                        {{
                            arg{index}Option = {helper}None();
                        }}
                        else
                        {{
                            IntPtr optionMap = {map_from}(arg{index});
                            try
                            {{
                                arg{index}Option = {helper}Some(optionMap);
                            }}
                            finally
                            {{
                                {map_drop}(optionMap);
                            }}
                        }}"#
                                    )
                                }
                                _ => unreachable!("unsupported C# option argument inner conversion"),
                            }
                        })
            .collect::<Vec<_>>();
        let vec_allocations = vec_arguments
                        .iter()
                        .map(|index| {
                            let ty = &arguments[*index];
                            let helper = c_sharp_vec_prefix(ty);
                            format!(
                                "arg{index}Vec = {helper}FromList(arg{index});\n            if (arg{index}Vec == IntPtr.Zero) throw new OutOfMemoryException();"
                            )
                        })
                        .collect::<Vec<_>>();
        let hashmap_allocations = hashmap_arguments
                        .iter()
                        .map(|index| {
                            let ty = &arguments[*index];
                            let helper = c_sharp_map_prefix(ty);
                            format!(
                                "arg{index}Map = {helper}FromDictionary(arg{index});\n            if (arg{index}Map == IntPtr.Zero) throw new OutOfMemoryException();"
                            )
                        })
                        .collect::<Vec<_>>();
        let trait_allocations = trait_arguments
                        .iter()
                        .map(|index| {
                            format!(
                                "arg{index}Callback = new {}Callback(arg{index});\n            arg{index}Bridge = arg{index}Callback.Bridge;",
                                arguments[*index].managed
                            )
                        })
                        .collect::<Vec<_>>();
        let allocations = string_allocations
            .into_iter()
            .chain(option_allocations)
            .chain(vec_allocations)
            .chain(hashmap_allocations)
            .chain(trait_allocations)
            .collect::<Vec<_>>()
            .join("\n            ");
        let string_cleanup = string_arguments.iter().map(|index| {
            format!("if (arg{index}Utf8 != IntPtr.Zero) Marshal.FreeCoTaskMem(arg{index}Utf8);")
        });
        let option_cleanup = option_arguments.iter().map(|index| {
            let helper = c_sharp_option_prefix(&arguments[*index]);
            format!("if (arg{index}Option != IntPtr.Zero) {helper}Drop(arg{index}Option);")
        });
        let vec_cleanup = vec_arguments.iter().map(|index| {
            let helper = c_sharp_vec_prefix(&arguments[*index]);
            format!("if (arg{index}Vec != IntPtr.Zero) {helper}Drop(arg{index}Vec);")
        });
        let hashmap_cleanup = hashmap_arguments.iter().map(|index| {
            let helper = c_sharp_map_prefix(&arguments[*index]);
            format!("if (arg{index}Map != IntPtr.Zero) {helper}Drop(arg{index}Map);")
        });
        let cleanup = option_cleanup
            .chain(string_cleanup)
            .chain(vec_cleanup)
            .chain(hashmap_cleanup)
            .chain(
                trait_arguments
                    .iter()
                    .map(|index| format!("arg{index}Callback?.ThrowIfFailed();")),
            )
            .collect::<Vec<_>>()
            .join("\n            ");
        format!(
            r#"{declarations}
        try
        {{
            {allocations}
            {function_body}
        }}
        finally
        {{
            {cleanup}
        }}"#
        )
    };
    let string_helpers = if matches!(
        return_type.conversion,
        CSharpConversion::StringReturn | CSharpConversion::OptionReturn
    ) && return_type
        .option_inner
        .as_deref()
        .is_none_or(|inner| matches!(inner.conversion, CSharpConversion::String))
    {
        let data_symbol = format!("{extern_function_name}_string_data");
        let len_symbol = format!("{extern_function_name}_string_len");
        let drop_symbol = format!("{extern_function_name}_string_drop");
        let return_option = matches!(return_type.conversion, CSharpConversion::OptionReturn);
        let data_symbol = if return_option {
            format!("{extern_function_name}_option_string_data")
        } else {
            data_symbol
        };
        let len_symbol = if return_option {
            format!("{extern_function_name}_option_string_len")
        } else {
            len_symbol
        };
        let drop_symbol = if return_option {
            format!("{extern_function_name}_option_string_drop")
        } else {
            drop_symbol
        };
        format!(
            r#"
    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_data", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {data_symbol}(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_len", CallingConvention = CallingConvention.Cdecl)]
    private static extern nuint {len_symbol}(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {drop_symbol}(IntPtr value);
"#
        )
    } else {
        String::new()
    };
    let has_string_data =
        matches!(return_type.conversion, CSharpConversion::StringReturn)
            || (matches!(return_type.conversion, CSharpConversion::OptionReturn)
                && return_type.option_inner.as_deref().is_some_and(|inner| {
                    matches!(inner.conversion, CSharpConversion::String)
                        || (matches!(inner.conversion, CSharpConversion::Vec)
                            && inner.vec_inner.as_deref().is_some_and(|inner| {
                                matches!(inner.conversion, CSharpConversion::String)
                            }))
                }))
            || arguments.iter().any(|ty| {
                matches!(ty.conversion, CSharpConversion::Vec)
                    && ty
                        .vec_inner
                        .as_deref()
                        .is_some_and(|inner| matches!(inner.conversion, CSharpConversion::String))
                    || (matches!(ty.conversion, CSharpConversion::Option)
                        && ty.option_inner.as_deref().is_some_and(|inner| {
                            matches!(inner.conversion, CSharpConversion::Vec)
                                && inner.vec_inner.as_deref().is_some_and(|inner| {
                                    matches!(inner.conversion, CSharpConversion::String)
                                })
                        }))
            })
            || (matches!(return_type.conversion, CSharpConversion::VecReturn)
                && return_type
                    .vec_inner
                    .as_deref()
                    .is_some_and(|inner| matches!(inner.conversion, CSharpConversion::String)))
            || arguments.iter().any(|ty| {
                matches!(ty.conversion, CSharpConversion::HashMap)
                    && (ty
                        .map_key
                        .as_deref()
                        .is_some_and(|inner| matches!(inner.conversion, CSharpConversion::String))
                        || ty.map_value.as_deref().is_some_and(|inner| {
                            matches!(inner.conversion, CSharpConversion::String)
                        }))
            })
            || (matches!(return_type.conversion, CSharpConversion::HashMapReturn)
                && (return_type
                    .map_key
                    .as_deref()
                    .is_some_and(|inner| matches!(inner.conversion, CSharpConversion::String))
                    || return_type.map_value.as_deref().is_some_and(|inner| {
                        matches!(inner.conversion, CSharpConversion::String)
                    })))
            || matches!(return_type.conversion, CSharpConversion::ResultReturn)
            || arguments.iter().any(csharp_type_contains_string)
            || csharp_type_contains_string(&return_type);
    let has_vector = !vec_arguments.is_empty()
        || !option_vec_arguments.is_empty()
        || !option_map_arguments.is_empty()
        || !hashmap_arguments.is_empty()
        || matches!(return_type.conversion, CSharpConversion::VecReturn)
        || (matches!(return_type.conversion, CSharpConversion::OptionReturn)
            && return_type.option_inner.as_deref().is_some_and(|inner| {
                matches!(
                    inner.conversion,
                    CSharpConversion::Vec | CSharpConversion::HashMap
                )
            }))
        || matches!(return_type.conversion, CSharpConversion::HashMapReturn)
        || (matches!(return_type.conversion, CSharpConversion::ResultReturn)
            && return_type.result_inner.as_deref().is_some_and(|inner| {
                matches!(inner.conversion, CSharpConversion::Option)
                    && inner
                        .option_inner
                        .as_deref()
                        .is_some_and(|inner| matches!(inner.conversion, CSharpConversion::Vec))
            }))
        || (matches!(return_type.conversion, CSharpConversion::ResultReturn)
            && return_type.result_inner.as_deref().is_some_and(|inner| {
                matches!(
                    inner.conversion,
                    CSharpConversion::Vec | CSharpConversion::HashMap
                )
            }));
    let imports = format!(
        "{}{}",
        if has_string_data {
            "using System.Text;\n"
        } else {
            ""
        },
        if has_vector {
            "using System.Collections.Generic;\n"
        } else {
            ""
        }
    );
    let option_helpers = String::new();
    let vec_helpers = String::new();
    let hashmap_helpers = String::new();
    let result_helpers = String::new();
    let class_kind = if host_class == "FfiModule" {
        "static"
    } else {
        "sealed"
    };
    let static_modifier = if is_static { "static " } else { "" };
    let source = format!(
        r#"using System;
using System.Runtime.InteropServices;
        using static CSharpReusableHelpers;
        {imports}

public {class_kind} partial class {host_class}
{{
    [DllImport("{library_name}", EntryPoint = "{symbol}", CallingConvention = CallingConvention.Cdecl)]
    private static extern {native_return} {symbol}({native_parameters});
{string_helpers}
{option_helpers}
{vec_helpers}
{hashmap_helpers}
{result_helpers}

    public {static_modifier}{managed_return} {method_name}({parameters})
    {{
        {body}
    }}
}}
"#,
        class_kind = class_kind,
        symbol = extern_function_name,
        native_return = return_type.native,
        managed_return = return_type.managed,
        option_helpers = option_helpers,
        vec_helpers = vec_helpers,
        hashmap_helpers = hashmap_helpers,
        result_helpers = result_helpers,
        static_modifier = static_modifier,
    );

    let output_dir = c_sharp_output_dir();
    create_file(source, output_dir.join(output_file));
}

struct CSharpType {
    managed: String,
    native: String,
    conversion: CSharpConversion,
    option_inner: Option<Box<CSharpType>>,
    option_abi_name: Option<String>,
    vec_inner: Option<Box<CSharpType>>,
    vec_abi_name: Option<String>,
    map_key: Option<Box<CSharpType>>,
    map_value: Option<Box<CSharpType>>,
    map_abi_name: Option<String>,
    result_inner: Option<Box<CSharpType>>,
    result_abi_name: Option<String>,
}

fn csharp_type_contains_string(ty: &CSharpType) -> bool {
    match ty.conversion {
        CSharpConversion::String | CSharpConversion::StringReturn => true,
        CSharpConversion::Option | CSharpConversion::OptionReturn => ty
            .option_inner
            .as_deref()
            .is_some_and(csharp_type_contains_string),
        CSharpConversion::Vec | CSharpConversion::VecReturn => ty
            .vec_inner
            .as_deref()
            .is_some_and(csharp_type_contains_string),
        CSharpConversion::HashMap | CSharpConversion::HashMapReturn => {
            ty.map_key
                .as_deref()
                .is_some_and(csharp_type_contains_string)
                || ty
                    .map_value
                    .as_deref()
                    .is_some_and(csharp_type_contains_string)
        }
        CSharpConversion::Result | CSharpConversion::ResultReturn => ty
            .result_inner
            .as_deref()
            .is_some_and(csharp_type_contains_string),
        CSharpConversion::TraitObject | CSharpConversion::TraitObjectReturn => false,
        _ => false,
    }
}

impl CSharpType {
    fn new(
        managed: impl Into<String>,
        native: impl Into<String>,
        conversion: CSharpConversion,
    ) -> Self {
        Self {
            managed: managed.into(),
            native: native.into(),
            conversion,
            option_inner: None,
            option_abi_name: None,
            vec_inner: None,
            vec_abi_name: None,
            map_key: None,
            map_value: None,
            map_abi_name: None,
            result_inner: None,
            result_abi_name: None,
        }
    }
}

#[derive(Clone, Copy)]
enum CSharpConversion {
    Identity,
    Bool,
    Void,
    String,
    StringReturn,
    StructArgument,
    StructReturn,
    Option,
    OptionReturn,
    Vec,
    VecReturn,
    HashMap,
    HashMapReturn,
    Enum,
    EnumReturn,
    Result,
    ResultReturn,
    TraitObject,
    TraitObjectReturn,
}

fn c_sharp_type(wrapper_type: &WrapperType) -> Option<CSharpType> {
    let identity = |ty| Some(CSharpType::new(ty, ty, CSharpConversion::Identity));
    match wrapper_type {
        WrapperType::IntegerNumber(ty) => match ty.as_str() {
            "i8" => identity("sbyte"),
            "u8" => identity("byte"),
            "i16" => identity("short"),
            "u16" => identity("ushort"),
            "i32" => identity("int"),
            "u32" => identity("uint"),
            "i64" => identity("long"),
            "u64" => identity("ulong"),
            "usize" => identity("nuint"),
            _ => None,
        },
        WrapperType::FloatingPointNumber(ty) => match ty.as_str() {
            "f32" => identity("float"),
            "f64" => identity("double"),
            _ => None,
        },
        WrapperType::Bool => Some(CSharpType::new("bool", "byte", CSharpConversion::Bool)),
        WrapperType::Enum(name) => Some(CSharpType::new(name, "int", CSharpConversion::Enum)),
        WrapperType::String => Some(CSharpType::new(
            "string",
            "IntPtr",
            CSharpConversion::String,
        )),
        WrapperType::Struct(name) => Some(CSharpType::new(
            name,
            name,
            CSharpConversion::StructArgument,
        )),
        WrapperType::Option(inner) => {
            let inner_type = c_sharp_type(inner)?;
            if !matches!(
                inner_type.conversion,
                CSharpConversion::Identity
                    | CSharpConversion::Bool
                    | CSharpConversion::Enum
                    | CSharpConversion::String
                    | CSharpConversion::StructArgument
                    | CSharpConversion::Vec
                    | CSharpConversion::HashMap
            ) {
                return None;
            }
            let mut option_type = CSharpType::new(
                format!("{}?", inner_type.managed),
                "IntPtr",
                CSharpConversion::Option,
            );
            option_type.option_inner = Some(Box::new(inner_type));
            option_type.option_abi_name = Some(inner.name().to_string());
            Some(option_type)
        }
        WrapperType::Vec(inner) => {
            let inner_type = c_sharp_type(inner)?;
            if !matches!(
                inner_type.conversion,
                CSharpConversion::Identity
                    | CSharpConversion::Bool
                    | CSharpConversion::Enum
                    | CSharpConversion::String
                    | CSharpConversion::StructArgument
                    | CSharpConversion::Option
                    | CSharpConversion::HashMap
            ) {
                return None;
            }
            if matches!(inner_type.conversion, CSharpConversion::Option)
                && !inner_type.option_inner.as_deref().is_some_and(|inner| {
                    matches!(inner.conversion, CSharpConversion::Identity)
                        || (matches!(inner.conversion, CSharpConversion::Vec)
                            && inner.vec_inner.as_deref().is_some_and(|inner| {
                                matches!(inner.conversion, CSharpConversion::Option)
                                    && inner.option_inner.as_deref().is_some_and(|inner| {
                                        matches!(inner.conversion, CSharpConversion::Identity)
                                    })
                            }))
                })
            {
                return None;
            }
            let mut vec_type = CSharpType::new(
                format!("IReadOnlyList<{}>", inner_type.managed),
                "IntPtr",
                CSharpConversion::Vec,
            );
            vec_type.vec_inner = Some(Box::new(inner_type));
            vec_type.vec_abi_name = Some(inner.name().to_string());
            Some(vec_type)
        }
        WrapperType::HashMap(key, value) => {
            let key_type = c_sharp_type(key)?;
            let value_type = c_sharp_type(value)?;
            if !matches!(
                key_type.conversion,
                CSharpConversion::Identity
                    | CSharpConversion::Bool
                    | CSharpConversion::Enum
                    | CSharpConversion::String
            ) || !matches!(
                value_type.conversion,
                CSharpConversion::Identity
                    | CSharpConversion::Bool
                    | CSharpConversion::Enum
                    | CSharpConversion::String
                    | CSharpConversion::StructArgument
                    | CSharpConversion::Vec
                    | CSharpConversion::Option
                    | CSharpConversion::HashMap
            ) {
                return None;
            }
            if matches!(value_type.conversion, CSharpConversion::Option)
                && !value_type
                    .option_inner
                    .as_deref()
                    .is_some_and(|inner| matches!(inner.conversion, CSharpConversion::Identity))
            {
                return None;
            }
            let mut map_type = CSharpType::new(
                format!("Dictionary<{}, {}>", key_type.managed, value_type.managed),
                "IntPtr",
                CSharpConversion::HashMap,
            );
            map_type.map_key = Some(Box::new(key_type));
            map_type.map_value = Some(Box::new(value_type));
            map_type.map_abi_name = Some(format!("hashmap_{}_{}", key.name(), value.name()));
            Some(map_type)
        }
        WrapperType::Result(inner) => {
            let inner_type = c_sharp_type(inner)?;
            if !matches!(
                inner_type.conversion,
                CSharpConversion::Identity
                    | CSharpConversion::Bool
                    | CSharpConversion::Void
                    | CSharpConversion::String
                    | CSharpConversion::Enum
                    | CSharpConversion::StructArgument
                    | CSharpConversion::Vec
                    | CSharpConversion::Option
                    | CSharpConversion::HashMap
            ) {
                return None;
            }
            let mut result_type = CSharpType::new(
                result_managed_type(&inner_type),
                "IntPtr",
                CSharpConversion::Result,
            );
            result_type.result_inner = Some(Box::new(inner_type));
            result_type.result_abi_name = Some(inner.name().to_string());
            Some(result_type)
        }
        WrapperType::UnitExpr => Some(CSharpType::new("void", "void", CSharpConversion::Void)),
        WrapperType::Trait(name) => Some(CSharpType::new(
            name,
            format!("{name}Bridge"),
            CSharpConversion::TraitObject,
        )),
    }
}

fn result_managed_type(inner: &CSharpType) -> String {
    match inner.conversion {
        CSharpConversion::Vec => format!(
            "List<{}>",
            inner
                .vec_inner
                .as_deref()
                .expect("vector result inner type")
                .managed
        ),
        _ => inner.managed.clone(),
    }
}

fn result_option_string_reader(library_name: &str, prefix: &str) -> String {
    format!(
        r#"
    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_data", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {prefix}ResultOptionStringData(IntPtr value);
    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_len", CallingConvention = CallingConvention.Cdecl)]
    private static extern nuint {prefix}ResultOptionStringLength(IntPtr value);
    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {prefix}ResultOptionStringDrop(IntPtr value);

    private static string ReadResultOptionString(IntPtr value)
    {{
        if (value == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null string inside an option result.");
        try
        {{
            nuint length = {prefix}ResultOptionStringLength(value);
            if (length > int.MaxValue) throw new OverflowException("Rust string is too large for a .NET string.");
            byte[] bytes = new byte[(int)length];
            if (bytes.Length > 0) Marshal.Copy({prefix}ResultOptionStringData(value), bytes, 0, bytes.Length);
            return Encoding.UTF8.GetString(bytes);
        }}
        finally
        {{
            {prefix}ResultOptionStringDrop(value);
        }}
    }}
"#
    )
}

fn result_helper_source(
    library_name: &str,
    prefix: &str,
    inner: &CSharpType,
    abi_name: &str,
) -> String {
    let drop = format!("{prefix}Drop");
    let unwrap = format!("{prefix}Unwrap");
    let unwrap_err = format!("{prefix}UnwrapErr");
    let is_err = format!("{prefix}IsErr");
    let arc_drop = format!("{prefix}ArcDrop");
    let arc_source = format!("{prefix}ArcSource");
    let error_description = format!("{prefix}ErrorDescription");
    let error_desc = format!("{prefix}ErrorDesc");
    let ref_drop = format!("{prefix}RefErrorDrop");
    let ref_desc = format!("{prefix}RefErrorDesc");
    let ref_source = format!("{prefix}RefErrorSource");
    let source_description = format!("{prefix}SourceErrorDescription");
    let create_exception = format!("{prefix}CreateException");
    let result_native = match inner.conversion {
        CSharpConversion::Identity => inner.native.as_str(),
        CSharpConversion::Bool => "byte",
        CSharpConversion::Enum => "int",
        CSharpConversion::Void => "void",
        CSharpConversion::String
        | CSharpConversion::StructArgument
        | CSharpConversion::Vec
        | CSharpConversion::Option
        | CSharpConversion::HashMap => "IntPtr",
        _ => unreachable!("unsupported C# result inner conversion"),
    };
    let value_string_helpers = if matches!(inner.conversion, CSharpConversion::String) {
        let data = format!("{prefix}StringData");
        let length = format!("{prefix}StringLength");
        let string_drop = format!("{prefix}StringDrop");
        let read = format!("{prefix}ReadString");
        format!(
            r#"
    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_data", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {data}(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_len", CallingConvention = CallingConvention.Cdecl)]
    private static extern nuint {length}(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {string_drop}(IntPtr value);

    private static string {read}(IntPtr value)
    {{
        if (value == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null string pointer.");
        try
        {{
            nuint byteLength = {length}(value);
            if (byteLength > int.MaxValue) throw new OverflowException("Rust string is too large for a .NET string.");
            byte[] bytes = new byte[(int)byteLength];
            if (bytes.Length > 0) Marshal.Copy({data}(value), bytes, 0, bytes.Length);
            return Encoding.UTF8.GetString(bytes);
        }}
        finally
        {{
            {string_drop}(value);
        }}
    }}
"#
        )
    } else {
        String::new()
    };
    format!(
        r#"
    [DllImport("{library_name}", EntryPoint = "hiFfi__drop_{abi_name}_result", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {drop}(IntPtr result);

    [DllImport("{library_name}", EntryPoint = "hiFfi__unwrap_{abi_name}_result", CallingConvention = CallingConvention.Cdecl)]
    private static extern {result_native} {unwrap}(IntPtr result);

    [DllImport("{library_name}", EntryPoint = "hiFfi__unwrap_err_{abi_name}_result", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {unwrap_err}(IntPtr result);

    [DllImport("{library_name}", EntryPoint = "hiFfi__is_err_{abi_name}_result", CallingConvention = CallingConvention.Cdecl)]
    private static extern byte {is_err}(IntPtr result);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_arc_dyn_err_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {arc_drop}(IntPtr error);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_arc_dyn_err_desc", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {error_desc}(IntPtr error);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_arc_dyn_err_source", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {arc_source}(IntPtr error);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_ref_dyn_err_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {ref_drop}(IntPtr error);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_ref_dyn_err_desc", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {ref_desc}(IntPtr error);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_ref_dyn_err_source", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {ref_source}(IntPtr error);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_data", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {prefix}ErrorStringData(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_len", CallingConvention = CallingConvention.Cdecl)]
    private static extern nuint {prefix}ErrorStringLength(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {prefix}ErrorStringDrop(IntPtr value);

    private static string {error_description}(IntPtr error)
    {{
        IntPtr description = {error_desc}(error);
        if (description == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null error description.");
        try
        {{
            nuint byteLength = {prefix}ErrorStringLength(description);
            if (byteLength > int.MaxValue) throw new OverflowException("Rust error description is too large for a .NET string.");
            byte[] bytes = new byte[(int)byteLength];
            if (bytes.Length > 0) Marshal.Copy({prefix}ErrorStringData(description), bytes, 0, bytes.Length);
            return Encoding.UTF8.GetString(bytes);
        }}
        finally
        {{
            {prefix}ErrorStringDrop(description);
        }}
    }}

    private static string {source_description}(IntPtr error)
    {{
        IntPtr description = {ref_desc}(error);
        if (description == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null error description.");
        try
        {{
            nuint byteLength = {prefix}ErrorStringLength(description);
            if (byteLength > int.MaxValue) throw new OverflowException("Rust error description is too large for a .NET string.");
            byte[] bytes = new byte[(int)byteLength];
            if (bytes.Length > 0) Marshal.Copy({prefix}ErrorStringData(description), bytes, 0, bytes.Length);
            return Encoding.UTF8.GetString(bytes);
        }}
        finally
        {{
            {prefix}ErrorStringDrop(description);
        }}
    }}

    private static Exception {create_exception}(IntPtr error)
    {{
        var descriptions = new System.Collections.Generic.List<string>
        {{
            {error_description}(error)
        }};
        IntPtr source = {arc_source}(error);
        while (source != IntPtr.Zero)
        {{
            IntPtr next = IntPtr.Zero;
            try
            {{
                descriptions.Add({source_description}(source));
                next = {ref_source}(source);
            }}
            finally
            {{
                {ref_drop}(source);
            }}
            source = next;
        }}

        Exception? inner = null;
        for (int index = descriptions.Count - 1; index >= 0; index--)
        {{
            inner = new InvalidOperationException(descriptions[index], inner);
        }}
        return inner!;
    }}
{value_string_helpers}
"#,
    )
}

fn option_helper_source(
    library_name: &str,
    prefix: &str,
    inner: &CSharpType,
    abi_name: &str,
) -> String {
    let some = format!("{prefix}Some");
    let none = format!("{prefix}None");
    let drop = format!("{prefix}Drop");
    let unwrap = format!("{prefix}Unwrap");
    let is_some = format!("{prefix}IsSome");
    let unwrap_native = if matches!(inner.conversion, CSharpConversion::Bool) {
        "byte"
    } else if matches!(
        inner.conversion,
        CSharpConversion::String | CSharpConversion::StructArgument
    ) {
        "IntPtr"
    } else {
        inner.native.as_str()
    };
    format!(
        r#"
    [DllImport("{library_name}", EntryPoint = "hiFfi__some_{abi_name}_option", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {some}({some_native} value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__none_{abi_name}_option", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {none}();

    [DllImport("{library_name}", EntryPoint = "hiFfi__drop_{abi_name}_option", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {drop}(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__unwrap_{abi_name}_option", CallingConvention = CallingConvention.Cdecl)]
    private static extern {unwrap_native} {unwrap}(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__is_some_{abi_name}_option", CallingConvention = CallingConvention.Cdecl)]
    private static extern byte {is_some}(IntPtr value);
"#,
        some_native = if matches!(
            inner.conversion,
            CSharpConversion::String | CSharpConversion::StructArgument
        ) {
            "IntPtr"
        } else {
            inner.native.as_str()
        }
    )
}

fn vec_helper_source(
    library_name: &str,
    prefix: &str,
    inner: &CSharpType,
    abi_name: &str,
) -> String {
    let drop = format!("{prefix}Drop");
    let with_capacity = format!("{prefix}WithCapacity");
    let push = format!("{prefix}Push");
    let len = format!("{prefix}Len");
    let get = format!("{prefix}Get");
    let push_arguments = match inner.conversion {
        CSharpConversion::Identity => format!("{} value", inner.native),
        CSharpConversion::Bool => "byte value".to_string(),
        CSharpConversion::Enum => "int value".to_string(),
        CSharpConversion::String => "IntPtr value, nuint valueLength".to_string(),
        CSharpConversion::StructArgument => "IntPtr value".to_string(),
        CSharpConversion::Option => "IntPtr value".to_string(),
        CSharpConversion::HashMap => "IntPtr value".to_string(),
        _ => unreachable!("unsupported C# vector element conversion"),
    };
    let get_native = match inner.conversion {
        CSharpConversion::Bool => "byte".to_string(),
        CSharpConversion::Enum => "int".to_string(),
        CSharpConversion::String
        | CSharpConversion::StructArgument
        | CSharpConversion::Option
        | CSharpConversion::HashMap => "IntPtr".to_string(),
        _ => inner.native.clone(),
    };
    let push_managed = match inner.conversion {
        CSharpConversion::String => format!(
            r#"if (value is null) throw new ArgumentNullException(nameof(value));
            if (value.IndexOf('\0') >= 0) throw new ArgumentException("Vector strings cannot contain NUL characters.", nameof(value));
            IntPtr utf8 = Marshal.StringToCoTaskMemUTF8(value);
            if (utf8 == IntPtr.Zero) throw new OutOfMemoryException();
            try
            {{
                {push}(vector, utf8, (nuint)Encoding.UTF8.GetByteCount(value));
            }}
            finally
            {{
                Marshal.FreeCoTaskMem(utf8);
            }}"#
        ),
        CSharpConversion::StructArgument => format!(
            r#"if (value is null) throw new ArgumentNullException(nameof(value));
            bool addedReference = false;
            value.DangerousAddRef(ref addedReference);
            try
            {{
                {push}(vector, value.DangerousGetHandle());
            }}
            finally
            {{
                if (addedReference) value.DangerousRelease();
            }}"#
        ),
        CSharpConversion::Option => {
            let option_prefix = c_sharp_option_prefix(inner);
            let some = format!("{option_prefix}Some");
            let none = format!("{option_prefix}None");
            let drop = format!("{option_prefix}Drop");
            let option_inner = inner
                .option_inner
                .as_deref()
                .expect("nested option inner type");
            if matches!(option_inner.conversion, CSharpConversion::Vec) {
                let option_vec_prefix = c_sharp_vec_prefix(option_inner);
                let from_list = format!("{option_vec_prefix}FromList");
                let vec_drop = format!("{option_vec_prefix}Drop");
                format!(
                    r#"IntPtr option;
            if (value is null)
            {{
                option = {none}();
            }}
            else
            {{
                IntPtr nestedVector = {from_list}(value);
                try
                {{
                    option = {some}(nestedVector);
                }}
                finally
                {{
                    {vec_drop}(nestedVector);
                }}
            }}
            try
            {{
                {push}(vector, option);
            }}
            finally
            {{
                {drop}(option);
            }}"#
                )
            } else {
                format!(
                    r#"IntPtr option = value.HasValue ? {some}(value.Value) : {none}();
            try
            {{
                {push}(vector, option);
            }}
            finally
            {{
                {drop}(option);
            }}"#
                )
            }
        }
        CSharpConversion::Bool => {
            format!("{push}(vector, value ? (byte)1 : (byte)0);")
        }
        CSharpConversion::Enum => format!("{push}(vector, (int)value);"),
        CSharpConversion::HashMap => {
            let map_prefix = c_sharp_map_prefix(inner);
            let from_dictionary = format!("{map_prefix}FromDictionary");
            let drop_map = format!("{map_prefix}Drop");
            format!(
                r#"IntPtr map = {from_dictionary}(value);
            try
            {{
                {push}(vector, map);
            }}
            finally
            {{
                {drop_map}(map);
            }}"#
            )
        }
        _ => format!("{push}(vector, value);"),
    };
    let get_managed = match inner.conversion {
        CSharpConversion::Bool => format!("{get}(vector, index) != 0"),
        CSharpConversion::Enum => {
            format!(
                "({managed_type}){get}(vector, index)",
                managed_type = inner.managed
            )
        }
        CSharpConversion::String => {
            let read_string = format!("{prefix}ReadString");
            format!("{read_string}({get}(vector, index))")
        }
        CSharpConversion::StructArgument => format!(
            "{managed_type}.FromNative({get}(vector, index))",
            managed_type = inner.managed
        ),
        CSharpConversion::Option => {
            format!("{prefix}ReadOption({get}(vector, index))")
        }
        CSharpConversion::HashMap => format!("{prefix}ReadMap({get}(vector, index))"),
        _ => format!("{get}(vector, index)"),
    };
    let nested_option_helpers = if matches!(inner.conversion, CSharpConversion::Option) {
        let option_inner = inner
            .option_inner
            .as_deref()
            .expect("nested option inner type");
        let option_prefix = c_sharp_option_prefix(inner);
        let read_option = if matches!(option_inner.conversion, CSharpConversion::Vec) {
            let option_vec_prefix = c_sharp_vec_prefix(option_inner);
            format!(
                r#"IntPtr nestedVector = {option_prefix}Unwrap(value);
            if (nestedVector == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null nested vector pointer.");
            try
            {{
                return {option_vec_prefix}ToList(nestedVector);
            }}
            finally
            {{
                {option_vec_prefix}Drop(nestedVector);
            }}"#
            )
        } else {
            format!("return {option_prefix}Unwrap(value);")
        };
        format!(
            r#"

    private static {}? {prefix}ReadOption(IntPtr value)
    {{
        if (value == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null option pointer in a vector.");
        try
        {{
            if ({option_prefix}IsSome(value) == 0) return null;
            {read_option}
        }}
        finally
        {{
            {option_prefix}Drop(value);
        }}
    }}
"#,
            option_inner.managed
        )
    } else {
        String::new()
    };
    let nested_map_helpers = if matches!(inner.conversion, CSharpConversion::HashMap) {
        let map_prefix = c_sharp_map_prefix(inner);
        let key = inner.map_key.as_deref().expect("vector map key type");
        let value = inner.map_value.as_deref().expect("vector map value type");
        format!(
            r#"
    private static Dictionary<{key_type}, {value_type}> {prefix}ReadMap(IntPtr map)
    {{
        if (map == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null HashMap pointer in a vector.");
        try
        {{
            return {map_prefix}ToDictionary(map);
        }}
        finally
        {{
            {map_prefix}Drop(map);
        }}
    }}
"#,
            key_type = key.managed,
            value_type = value.managed,
        )
    } else {
        String::new()
    };
    let string_helpers = if matches!(inner.conversion, CSharpConversion::String) {
        let data = format!("{prefix}StringData");
        let string_len = format!("{prefix}StringLen");
        let string_drop = format!("{prefix}StringDrop");
        let read_string = format!("{prefix}ReadString");
        format!(
            r#"
    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_data", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {data}(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_len", CallingConvention = CallingConvention.Cdecl)]
    private static extern nuint {string_len}(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {string_drop}(IntPtr value);

    private static string {read_string}(IntPtr value)
    {{
        if (value == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null string pointer.");
        try
        {{
            nuint byteLength = {string_len}(value);
            if (byteLength > int.MaxValue) throw new OverflowException("Rust string is too large for a .NET string.");
            byte[] bytes = new byte[(int)byteLength];
            if (bytes.Length > 0) Marshal.Copy({data}(value), bytes, 0, bytes.Length);
            return Encoding.UTF8.GetString(bytes);
        }}
        finally
        {{
            {string_drop}(value);
        }}
    }}
"#
        )
    } else {
        String::new()
    };
    format!(
        r#"
    [DllImport("{library_name}", EntryPoint = "hiFfi__drop_{abi_name}_vec", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {drop}(IntPtr vector);

    [DllImport("{library_name}", EntryPoint = "hiFfi__with_capacity_{abi_name}_vec", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {with_capacity}(nuint capacity);

    [DllImport("{library_name}", EntryPoint = "hiFfi__push_{abi_name}_vec", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {push}(IntPtr vector, {push_arguments});

    [DllImport("{library_name}", EntryPoint = "hiFfi__len_{abi_name}_vec", CallingConvention = CallingConvention.Cdecl)]
    private static extern nuint {len}(IntPtr vector);

    [DllImport("{library_name}", EntryPoint = "hiFfi__get_{abi_name}_vec", CallingConvention = CallingConvention.Cdecl)]
    private static extern {get_native} {get}(IntPtr vector, nuint index);
{string_helpers}
{nested_option_helpers}
{nested_map_helpers}

    private static IntPtr {prefix}FromList(IReadOnlyList<{managed_type}> values)
    {{
        if (values is null) throw new ArgumentNullException(nameof(values));
        IntPtr vector = {with_capacity}((nuint)values.Count);
        if (vector == IntPtr.Zero) throw new OutOfMemoryException();
        try
        {{
            foreach (var value in values)
            {{
                {push_managed}
            }}
            return vector;
        }}
        catch
        {{
            {drop}(vector);
            throw;
        }}
    }}

    private static List<{managed_type}> {prefix}ToList(IntPtr vector)
    {{
        nuint length = {len}(vector);
        if (length > int.MaxValue) throw new OverflowException("Rust vector is too large for a .NET list.");
        var values = new List<{managed_type}>((int)length);
        for (nuint index = 0; index < length; index++)
        {{
            values.Add({get_managed});
        }}
        return values;
    }}
"#,
        managed_type = inner.managed,
    )
}

fn hashmap_helper_source(
    library_name: &str,
    prefix: &str,
    key: &CSharpType,
    value: &CSharpType,
    abi_name: &str,
) -> String {
    let new = format!("{prefix}New");
    let drop = format!("{prefix}Drop");
    let insert = format!("{prefix}Insert");
    let len = format!("{prefix}Len");
    let key_at = format!("{prefix}KeyAt");
    let value_at = format!("{prefix}ValueAt");
    let key_native = hashmap_native_type(key);
    let value_native = hashmap_native_type(value);
    let (key_signature, key_setup, key_argument, key_cleanup) =
        hashmap_insert_argument(key, "key", prefix);
    let (value_signature, value_setup, value_argument, value_cleanup) =
        hashmap_insert_argument(value, "value", prefix);
    let insert_body = format!("{insert}(map, {key_argument}, {value_argument});");
    let insert_body = hashmap_with_resource(
        &key_setup,
        &key_cleanup,
        &hashmap_with_resource(&value_setup, &value_cleanup, &insert_body),
    );
    let key_read = hashmap_read_expression(key, &key_at, prefix);
    let value_read = hashmap_read_expression(value, &value_at, prefix);
    let nested_value_helpers = match value.conversion {
        CSharpConversion::Vec => {
            let helper = c_sharp_vec_prefix(value);
            let inner = value.vec_inner.as_deref().expect("map vector element type");
            format!(
                r#"
    private static List<{element_type}> {prefix}ReadValueVec(IntPtr vector)
    {{
        if (vector == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null nested HashMap vector.");
        try
        {{
            return {helper}ToList(vector);
        }}
        finally
        {{
            {helper}Drop(vector);
        }}
    }}
"#,
                element_type = inner.managed,
            )
        }
        CSharpConversion::Option => {
            let helper = c_sharp_option_prefix(value);
            let inner = value
                .option_inner
                .as_deref()
                .expect("map option inner type");
            format!(
                r#"
    private static {}? {prefix}ReadValueOption(IntPtr option)
    {{
        if (option == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null nested HashMap option.");
        try
        {{
            if ({helper}IsSome(option) == 0) return null;
            return {helper}Unwrap(option);
        }}
        finally
        {{
            {helper}Drop(option);
        }}
    }}
"#,
                inner.managed
            )
        }
        CSharpConversion::HashMap => {
            let helper = c_sharp_map_prefix(value);
            let key = value.map_key.as_deref().expect("nested map key type");
            let inner_value = value.map_value.as_deref().expect("nested map value type");
            format!(
                r#"
    private static Dictionary<{key_type}, {value_type}> {prefix}ReadValueMap(IntPtr map)
    {{
        if (map == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null nested HashMap.");
        try
        {{
            return {helper}ToDictionary(map);
        }}
        finally
        {{
            {helper}Drop(map);
        }}
    }}
"#,
                key_type = key.managed,
                value_type = inner_value.managed,
            )
        }
        _ => String::new(),
    };
    let string_helpers = if matches!(key.conversion, CSharpConversion::String)
        || matches!(value.conversion, CSharpConversion::String)
    {
        hashmap_string_helpers(library_name, prefix)
    } else {
        String::new()
    };
    format!(
        r#"
    [DllImport("{library_name}", EntryPoint = "hiFfi__new_{abi_name}", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {new}();

    [DllImport("{library_name}", EntryPoint = "hiFfi__drop_{abi_name}", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {drop}(IntPtr map);

    [DllImport("{library_name}", EntryPoint = "hiFfi__insert_{abi_name}", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {insert}(IntPtr map, {key_signature}, {value_signature});

    [DllImport("{library_name}", EntryPoint = "hiFfi__len_{abi_name}", CallingConvention = CallingConvention.Cdecl)]
    private static extern nuint {len}(IntPtr map);

    [DllImport("{library_name}", EntryPoint = "hiFfi__key_at_{abi_name}", CallingConvention = CallingConvention.Cdecl)]
    private static extern {key_native} {key_at}(IntPtr map, nuint index);

    [DllImport("{library_name}", EntryPoint = "hiFfi__value_at_{abi_name}", CallingConvention = CallingConvention.Cdecl)]
    private static extern {value_native} {value_at}(IntPtr map, nuint index);
{string_helpers}
{nested_value_helpers}

    private static IntPtr {prefix}FromDictionary(IDictionary<{key_type}, {value_type}> values)
    {{
        if (values is null) throw new ArgumentNullException(nameof(values));
        IntPtr map = {new}();
        if (map == IntPtr.Zero) throw new OutOfMemoryException();
        try
        {{
            foreach (var pair in values)
            {{
                var key = pair.Key;
                var value = pair.Value;
                {insert_body}
            }}
            return map;
        }}
        catch
        {{
            {drop}(map);
            throw;
        }}
    }}

    private static Dictionary<{key_type}, {value_type}> {prefix}ToDictionary(IntPtr map)
    {{
        nuint length = {len}(map);
        if (length > int.MaxValue) throw new OverflowException("Rust HashMap is too large for a .NET dictionary.");
        var values = new Dictionary<{key_type}, {value_type}>((int)length);
        for (nuint index = 0; index < length; index++)
        {{
            var key = {key_read};
            var value = {value_read};
            values.Add(key, value);
        }}
        return values;
    }}
"#,
        key_type = key.managed,
        value_type = value.managed,
    )
}

fn hashmap_native_type(ty: &CSharpType) -> &'static str {
    match ty.conversion {
        CSharpConversion::Bool => "byte",
        CSharpConversion::Enum => "int",
        CSharpConversion::String
        | CSharpConversion::StructArgument
        | CSharpConversion::Vec
        | CSharpConversion::Option
        | CSharpConversion::HashMap => "IntPtr",
        CSharpConversion::Identity => match ty.native.as_str() {
            "sbyte" => "sbyte",
            "byte" => "byte",
            "short" => "short",
            "ushort" => "ushort",
            "int" => "int",
            "uint" => "uint",
            "long" => "long",
            "ulong" => "ulong",
            "nuint" => "nuint",
            "float" => "float",
            "double" => "double",
            _ => unreachable!("unsupported C# HashMap element type"),
        },
        _ => unreachable!("unsupported C# HashMap element conversion"),
    }
}

fn hashmap_insert_argument(
    ty: &CSharpType,
    name: &str,
    prefix: &str,
) -> (String, String, String, String) {
    match ty.conversion {
        CSharpConversion::String => {
            let pointer = format!("{prefix}{name}Utf8");
            (
                format!("IntPtr {name}, nuint {name}Length"),
                format!("IntPtr {pointer} = {prefix}EncodeString({name});"),
                format!("{pointer}, (nuint)Encoding.UTF8.GetByteCount({name})"),
                format!("Marshal.FreeCoTaskMem({pointer});"),
            )
        }
        CSharpConversion::Bool => (
            format!("byte {name}"),
            String::new(),
            format!("{name} ? (byte)1 : (byte)0"),
            String::new(),
        ),
        CSharpConversion::Enum => (
            format!("int {name}"),
            String::new(),
            format!("(int){name}"),
            String::new(),
        ),
        CSharpConversion::StructArgument => (
            format!("IntPtr {name}"),
            format!(
                "if ({name} is null) throw new ArgumentNullException(nameof({name}));\n                bool {name}AddedReference = false;\n                {name}.DangerousAddRef(ref {name}AddedReference);"
            ),
            format!("{name}.DangerousGetHandle()"),
            format!("if ({name}AddedReference) {name}.DangerousRelease();"),
        ),
        CSharpConversion::Vec => {
            let helper = c_sharp_vec_prefix(ty);
            (
                "IntPtr value".to_string(),
                format!("IntPtr valueContainer = {helper}FromList(value);"),
                "valueContainer".to_string(),
                format!("{helper}Drop(valueContainer);"),
            )
        }
        CSharpConversion::Option => {
            let helper = c_sharp_option_prefix(ty);
            (
                "IntPtr value".to_string(),
                format!(
                    "IntPtr valueOption = value.HasValue ? {helper}Some(value.Value) : {helper}None();"
                ),
                "valueOption".to_string(),
                format!("{helper}Drop(valueOption);"),
            )
        }
        CSharpConversion::HashMap => {
            let helper = c_sharp_map_prefix(ty);
            (
                "IntPtr value".to_string(),
                format!("IntPtr valueMap = {helper}FromDictionary(value);"),
                "valueMap".to_string(),
                format!("{helper}Drop(valueMap);"),
            )
        }
        CSharpConversion::Identity => (
            format!("{} {name}", ty.native),
            String::new(),
            name.to_string(),
            String::new(),
        ),
        _ => unreachable!("unsupported C# HashMap insertion type"),
    }
}

fn hashmap_with_resource(setup: &str, cleanup: &str, body: &str) -> String {
    if setup.is_empty() {
        return body.to_string();
    }
    format!(
        r#"{setup}
                try
                {{
                    {body}
                }}
                finally
                {{
                    {cleanup}
                }}"#
    )
}

fn hashmap_read_expression(ty: &CSharpType, function: &str, prefix: &str) -> String {
    match ty.conversion {
        CSharpConversion::Bool => format!("{function}(map, index) != 0"),
        CSharpConversion::Enum => {
            format!("({}){function}(map, index)", ty.managed)
        }
        CSharpConversion::String => format!("{prefix}ReadString({function}(map, index))"),
        CSharpConversion::StructArgument => {
            format!("{}.FromNative({function}(map, index))", ty.managed)
        }
        CSharpConversion::Vec => format!("{prefix}ReadValueVec({function}(map, index))"),
        CSharpConversion::Option => format!("{prefix}ReadValueOption({function}(map, index))"),
        CSharpConversion::HashMap => format!("{prefix}ReadValueMap({function}(map, index))"),
        _ => format!("{function}(map, index)"),
    }
}

fn hashmap_string_helpers(library_name: &str, prefix: &str) -> String {
    format!(
        r#"
    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_data", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {prefix}StringData(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_len", CallingConvention = CallingConvention.Cdecl)]
    private static extern nuint {prefix}StringLength(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {prefix}StringDrop(IntPtr value);

    private static IntPtr {prefix}EncodeString(string value)
    {{
        if (value is null) throw new ArgumentNullException(nameof(value));
        if (value.IndexOf('\0') >= 0) throw new ArgumentException("HashMap strings cannot contain NUL characters.", nameof(value));
        IntPtr utf8 = Marshal.StringToCoTaskMemUTF8(value);
        if (utf8 == IntPtr.Zero) throw new OutOfMemoryException();
        return utf8;
    }}

    private static string {prefix}ReadString(IntPtr value)
    {{
        if (value == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null HashMap string pointer.");
        try
        {{
            nuint byteLength = {prefix}StringLength(value);
            if (byteLength > int.MaxValue) throw new OverflowException("Rust HashMap string is too large for a .NET string.");
            byte[] bytes = new byte[(int)byteLength];
            if (bytes.Length > 0) Marshal.Copy({prefix}StringData(value), bytes, 0, bytes.Length);
            return Encoding.UTF8.GetString(bytes);
        }}
        finally
        {{
            {prefix}StringDrop(value);
        }}
    }}
"#
    )
}

fn write_c_sharp_enum(enum_wrapper: &EnumWrapper) {
    let variants = enum_wrapper
        .variants
        .iter()
        .map(|variant| match &variant.discriminant {
            Some(value) => format!("    {} = {value},", variant.name),
            None => format!("    {},", variant.name),
        })
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!(
        "public enum {} : int\n{{\n{}\n}}\n",
        enum_wrapper.name, variants
    );
    create_file(
        source,
        c_sharp_output_dir().join(format!("{}.cs", enum_wrapper.name)),
    );
}

fn c_sharp_vec_field_source(
    library_name: &str,
    struct_name: &str,
    field: &FieldWrapper,
    field_type: &CSharpType,
) -> (String, String, String, bool) {
    let field_name = to_pascal_case(&field.field_name.to_string());
    let prefix = c_sharp_vec_prefix(field_type);
    let inner = field_type
        .vec_inner
        .as_deref()
        .expect("vector field inner type");
    let mut imports = Vec::new();
    let mut accessors = Vec::new();

    if let Some(getter) = &field.getter {
        let method_name = to_pascal_case(&getter.name.to_string());
        let native_method = format!("{method_name}Native");
        imports.push(format!(
            r#"    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {native_method}({struct_name} instance);
"#,
            getter.extern_fn_name
        ));
        accessors.push(format!("get => {struct_name}Native.{method_name}(this);"));
        imports.push(format!(
            r#"    internal static List<{element_type}> {method_name}({struct_name} instance)
    {{
        bool addedReference = false;
        instance.DangerousAddRef(ref addedReference);
        try
        {{
            IntPtr vector = {native_method}(instance);
            if (vector == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null vector field pointer.");
            return {prefix}ToList(vector);
        }}
        finally
        {{
            if (addedReference) instance.DangerousRelease();
        }}
    }}
"#,
            element_type = inner.managed
        ));
    }
    if let Some(setter) = &field.setter {
        let method_name = to_pascal_case(&setter.name.to_string());
        let native_method = format!("{method_name}Native");
        imports.push(format!(
            r#"    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {native_method}({struct_name} instance, IntPtr value);
"#,
            setter.extern_fn_name
        ));
        imports.push(format!(
            r#"    internal static void {method_name}({struct_name} instance, IReadOnlyList<{element_type}> value)
    {{
        IntPtr vector = {prefix}FromList(value);
        try
        {{
            {native_method}(instance, vector);
        }}
        finally
        {{
            {prefix}Drop(vector);
        }}
    }}
"#,
            element_type = inner.managed
        ));
        accessors.push(format!(
            "set => {struct_name}Native.{method_name}(this, value);"
        ));
    }
    let property = format!(
        "    public List<{}> {field_name} {{ {} }}",
        inner.managed,
        accessors.join(" ")
    );
    let helpers = String::new();
    (
        imports.join("\n"),
        helpers,
        property,
        csharp_type_contains_string(inner),
    )
}

fn c_sharp_option_field_source(
    library_name: &str,
    struct_name: &str,
    field: &FieldWrapper,
    field_type: &CSharpType,
) -> (String, String, String, bool) {
    let field_name = to_pascal_case(&field.field_name.to_string());
    let prefix = format!("{struct_name}{field_name}OptionField");
    let reusable_prefix = c_sharp_option_prefix(field_type);
    let inner = field_type
        .option_inner
        .as_deref()
        .expect("option field inner type");
    let mut imports = Vec::new();
    let mut accessors = Vec::new();
    let vector_helpers = if matches!(inner.conversion, CSharpConversion::Vec) {
        Some((c_sharp_vec_prefix(inner), String::new()))
    } else {
        None
    };
    let map_helpers = if matches!(inner.conversion, CSharpConversion::HashMap) {
        Some((c_sharp_map_prefix(inner), String::new()))
    } else {
        None
    };

    if let Some(getter) = &field.getter {
        let method_name = to_pascal_case(&getter.name.to_string());
        let native_method = format!("{method_name}Native");
        let is_some = format!("{reusable_prefix}IsSome");
        let unwrap = format!("{reusable_prefix}Unwrap");
        let decode = match inner.conversion {
            CSharpConversion::Identity => format!("return {unwrap}(option);"),
            CSharpConversion::Bool => format!("return {unwrap}(option) != 0;"),
            CSharpConversion::Enum => {
                format!("return ({}){unwrap}(option);", inner.managed)
            }
            CSharpConversion::String => {
                let read = format!("{prefix}ReadString");
                format!("return {read}({unwrap}(option));")
            }
            CSharpConversion::StructArgument => format!(
                "return {managed_type}.FromNative({unwrap}(option));",
                managed_type = inner.managed
            ),
            CSharpConversion::Vec => {
                let (vector_prefix, _) = vector_helpers.as_ref().unwrap();
                format!(
                    r#"IntPtr vector = {unwrap}(option);
            if (vector == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null vector pointer in an option field.");
            try
            {{
                return {vector_prefix}ToList(vector);
            }}
            finally
            {{
                {vector_prefix}Drop(vector);
            }}"#
                )
            }
            CSharpConversion::HashMap => {
                let (map_prefix, _) = map_helpers.as_ref().unwrap();
                format!(
                    r#"IntPtr map = {unwrap}(option);
            if (map == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null HashMap pointer in an option field.");
            try
            {{
                return {map_prefix}ToDictionary(map);
            }}
            finally
            {{
                {map_prefix}Drop(map);
            }}"#
                )
            }
            _ => unreachable!("unsupported C# option field inner conversion"),
        };
        imports.push(format!(
            r#"    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {native_method}({struct_name} instance);

    internal static {managed_type} {method_name}({struct_name} instance)
    {{
        bool addedReference = false;
        instance.DangerousAddRef(ref addedReference);
        try
        {{
            IntPtr option = {native_method}(instance);
            if (option == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null option field pointer.");
            if ({is_some}(option) == 0) return null;
            {decode}
        }}
        finally
        {{
            if (addedReference) instance.DangerousRelease();
        }}
    }}
"#,
            getter.extern_fn_name,
            managed_type = field_type.managed
        ));
        if matches!(inner.conversion, CSharpConversion::String) {
            imports.push(option_field_string_helpers(library_name, &prefix));
        }
        accessors.push(format!("get => {struct_name}Native.{method_name}(this);"));
    }

    if let Some(setter) = &field.setter {
        let method_name = to_pascal_case(&setter.name.to_string());
        let native_method = format!("{method_name}Native");
        let some = format!("{reusable_prefix}Some");
        let none = format!("{reusable_prefix}None");
        let drop = format!("{reusable_prefix}Drop");
        let construct = match inner.conversion {
            CSharpConversion::Identity => {
                format!("value.HasValue ? {some}(value.Value) : {none}()")
            }
            CSharpConversion::Bool => {
                format!("value.HasValue ? {some}(value.Value ? (byte)1 : (byte)0) : {none}()")
            }
            CSharpConversion::Enum => {
                format!("value.HasValue ? {some}((int)value.Value) : {none}()")
            }
            CSharpConversion::String => format!("{prefix}BuildOptionString(value)"),
            CSharpConversion::StructArgument => {
                format!("value is null ? {none}() : {prefix}BuildOptionStruct(value)")
            }
            CSharpConversion::Vec => {
                vector_helpers.as_ref().expect("option vector field helper");
                format!(r#"value is null ? {none}() : {prefix}BuildOptionVector(value)"#)
            }
            CSharpConversion::HashMap => {
                map_helpers.as_ref().expect("option HashMap field helper");
                format!(r#"value is null ? {none}() : {prefix}BuildOptionHashMap(value)"#)
            }
            _ => unreachable!("unsupported C# option field inner conversion"),
        };
        imports.push(format!(
            r#"    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {native_method}({struct_name} instance, IntPtr value);

    internal static void {method_name}({struct_name} instance, {managed_type} value)
    {{
        IntPtr option = {construct};
        try
        {{
            {native_method}(instance, option);
        }}
        finally
        {{
            {drop}(option);
        }}
    }}
"#,
            setter.extern_fn_name,
            managed_type = field_type.managed
        ));
        if matches!(inner.conversion, CSharpConversion::StructArgument) {
            imports.push(format!(
                r#"    private static IntPtr {prefix}BuildOptionStruct({} value)
    {{
        bool addedReference = false;
        value.DangerousAddRef(ref addedReference);
        try {{ return {some}(value.DangerousGetHandle()); }}
        finally {{ if (addedReference) value.DangerousRelease(); }}
    }}
"#,
                inner.managed
            ));
        }
        if matches!(inner.conversion, CSharpConversion::String) {
            imports.push(option_field_string_setter_helpers(
                &prefix,
                &reusable_prefix,
            ));
        }
        if matches!(inner.conversion, CSharpConversion::Vec) {
            let (vector_prefix, _) = vector_helpers.as_ref().unwrap();
            imports.push(format!(
                r#"    private static IntPtr {prefix}BuildOptionVector(IReadOnlyList<{element_type}> value)
    {{
        IntPtr vector = {vector_prefix}FromList(value);
        try
        {{
            return {some}(vector);
        }}
        finally
        {{
            {vector_prefix}Drop(vector);
        }}
    }}
"#,
                element_type = inner
                    .vec_inner
                    .as_deref()
                    .expect("option vector element type")
                    .managed
            ));
        }
        if matches!(inner.conversion, CSharpConversion::HashMap) {
            let (map_prefix, _) = map_helpers.as_ref().unwrap();
            imports.push(format!(
                r#"    private static IntPtr {prefix}BuildOptionHashMap(IDictionary<{key_type}, {value_type}> value)
    {{
        IntPtr map = {map_prefix}FromDictionary(value);
        try
        {{
            return {some}(map);
        }}
        finally
        {{
            {map_prefix}Drop(map);
        }}
    }}
"#,
                key_type = inner.map_key.as_deref().expect("option map key type").managed,
                value_type = inner.map_value.as_deref().expect("option map value type").managed,
            ));
        }
        accessors.push(format!(
            "set => {struct_name}Native.{method_name}(this, value);"
        ));
    }

    (
        imports.join("\n"),
        vector_helpers.map_or_else(String::new, |(_, helpers)| helpers)
            + &map_helpers.map_or_else(String::new, |(_, helpers)| helpers),
        format!(
            "    public {} {field_name} {{ {} }}",
            field_type.managed,
            accessors.join(" ")
        ),
        matches!(inner.conversion, CSharpConversion::String),
    )
}

fn c_sharp_hashmap_field_source(
    library_name: &str,
    struct_name: &str,
    field: &FieldWrapper,
    field_type: &CSharpType,
) -> (String, String, String, bool) {
    let field_name = to_pascal_case(&field.field_name.to_string());
    let prefix = c_sharp_map_prefix(field_type);
    let key = field_type.map_key.as_deref().expect("map field key type");
    let value = field_type
        .map_value
        .as_deref()
        .expect("map field value type");
    let mut imports = Vec::new();
    let mut accessors = Vec::new();

    if let Some(getter) = &field.getter {
        let method_name = to_pascal_case(&getter.name.to_string());
        let native_method = format!("{method_name}Native");
        imports.push(format!(
            r#"    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {native_method}({struct_name} instance);

    internal static Dictionary<{key_type}, {value_type}> {method_name}({struct_name} instance)
    {{
        bool addedReference = false;
        instance.DangerousAddRef(ref addedReference);
        try
        {{
            IntPtr map = {native_method}(instance);
            if (map == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null HashMap field pointer.");
            return {prefix}ToDictionary(map);
        }}
        finally
        {{
            if (addedReference) instance.DangerousRelease();
        }}
    }}
"#,
            getter.extern_fn_name,
            key_type = key.managed,
            value_type = value.managed,
        ));
        accessors.push(format!("get => {struct_name}Native.{method_name}(this);"));
    }

    if let Some(setter) = &field.setter {
        let method_name = to_pascal_case(&setter.name.to_string());
        let native_method = format!("{method_name}Native");
        imports.push(format!(
            r#"    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {native_method}({struct_name} instance, IntPtr value);

    internal static void {method_name}({struct_name} instance, IDictionary<{key_type}, {value_type}> value)
    {{
        IntPtr map = {prefix}FromDictionary(value);
        try
        {{
            {native_method}(instance, map);
        }}
        finally
        {{
            {prefix}Drop(map);
        }}
    }}
"#,
            setter.extern_fn_name,
            key_type = key.managed,
            value_type = value.managed,
        ));
        accessors.push(format!(
            "set => {struct_name}Native.{method_name}(this, value);"
        ));
    }

    let has_string = csharp_type_contains_string(key) || csharp_type_contains_string(value);
    (
        imports.join("\n"),
        String::new(),
        format!(
            "    public {} {field_name} {{ {} }}",
            field_type.managed,
            accessors.join(" ")
        ),
        has_string,
    )
}

fn option_field_string_helpers(library_name: &str, prefix: &str) -> String {
    let data = format!("{prefix}StringData");
    let length = format!("{prefix}StringLength");
    let drop = format!("{prefix}StringDrop");
    let read = format!("{prefix}ReadString");
    format!(
        r#"
    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_data", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {data}(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_len", CallingConvention = CallingConvention.Cdecl)]
    private static extern nuint {length}(IntPtr value);

    [DllImport("{library_name}", EntryPoint = "hiFfi__rust_string_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {drop}(IntPtr value);

    private static string {read}(IntPtr value)
    {{
        if (value == IntPtr.Zero) throw new InvalidOperationException("Rust returned a null option string.");
        try
        {{
            nuint byteLength = {length}(value);
            if (byteLength > int.MaxValue) throw new OverflowException("Rust string field is too large for a .NET string.");
            byte[] bytes = new byte[(int)byteLength];
            if (bytes.Length > 0) Marshal.Copy({data}(value), bytes, 0, bytes.Length);
            return Encoding.UTF8.GetString(bytes);
        }}
        finally
        {{
            {drop}(value);
        }}
    }}
"#
    )
}

fn option_field_string_setter_helpers(prefix: &str, reusable_prefix: &str) -> String {
    let some = format!("{reusable_prefix}Some");
    let none = format!("{reusable_prefix}None");
    format!(
        r#"
    private static IntPtr {prefix}BuildOptionString(string? value)
    {{
        if (value is null) return {none}();
        if (value.IndexOf('\0') >= 0) throw new ArgumentException("Option strings cannot contain NUL characters.", nameof(value));
        IntPtr utf8 = Marshal.StringToCoTaskMemUTF8(value);
        if (utf8 == IntPtr.Zero) throw new OutOfMemoryException();
        try
        {{
            return {some}(utf8);
        }}
        finally
        {{
            Marshal.FreeCoTaskMem(utf8);
        }}
    }}
"#,
        prefix = prefix,
        some = some,
        none = none,
    )
}

fn write_c_sharp_struct(struct_wrapper: &StructWrapper) {
    let struct_name = struct_wrapper.name.to_string();
    let library_name = std::env::var("CARGO_PKG_NAME")
        .expect("Cargo package name expected")
        .replace('-', "_");
    let mut field_imports = Vec::new();
    let mut field_helpers = Vec::new();
    let mut properties = Vec::new();
    let mut has_string_field = false;
    let mut has_string_getter = false;
    let mut has_collection_field = false;
    for field in &struct_wrapper.fields {
        let Some(field_type) = c_sharp_type(&field.wrapper_type) else {
            continue;
        };
        if matches!(field_type.conversion, CSharpConversion::Vec) {
            let (imports, helpers, property, has_string) =
                c_sharp_vec_field_source(&library_name, &struct_name, field, &field_type);
            field_imports.push(imports);
            field_helpers.push(helpers);
            properties.push(property);
            has_string_field |= has_string;
            has_collection_field = true;
            continue;
        }
        if matches!(field_type.conversion, CSharpConversion::Option) {
            has_collection_field |= field_type.option_inner.as_deref().is_some_and(|inner| {
                matches!(
                    inner.conversion,
                    CSharpConversion::Vec | CSharpConversion::HashMap
                )
            });
            let (imports, helpers, property, has_string) =
                c_sharp_option_field_source(&library_name, &struct_name, field, &field_type);
            field_imports.push(imports);
            field_helpers.push(helpers);
            properties.push(property);
            has_string_field |= has_string;
            continue;
        }
        if matches!(field_type.conversion, CSharpConversion::HashMap) {
            let (imports, helpers, property, has_string) =
                c_sharp_hashmap_field_source(&library_name, &struct_name, field, &field_type);
            field_imports.push(imports);
            field_helpers.push(helpers);
            properties.push(property);
            has_string_field |= has_string;
            has_collection_field = true;
            continue;
        }
        let is_string = matches!(field_type.conversion, CSharpConversion::String);
        if !is_string
            && !matches!(
                field_type.conversion,
                CSharpConversion::Identity | CSharpConversion::Bool | CSharpConversion::Enum
            )
        {
            continue;
        }

        let field_name = to_pascal_case(&field.field_name.to_string());
        let mut accessors = Vec::new();
        if let Some(getter) = &field.getter {
            let method_name = to_pascal_case(&getter.name.to_string());
            if is_string {
                has_string_field = true;
                has_string_getter = true;
                let native_method = format!("{method_name}Native");
                field_imports.push(format!(
                    r#"    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr {native_method}({struct_name} value);
"#,
                    getter.extern_fn_name
                ));
                field_helpers.push(format!(
                    r#"    internal static string {method_name}({struct_name} instance)
    {{
        bool addedReference = false;
        instance.DangerousAddRef(ref addedReference);
        try
        {{
            IntPtr slice = {native_method}(instance);
            if (slice == IntPtr.Zero)
            {{
                throw new InvalidOperationException("Rust returned a null string slice.");
            }}
            try
            {{
                nuint byteLength = SliceLength(slice);
                if (byteLength > int.MaxValue)
                {{
                    throw new OverflowException("Rust string field is too large for a .NET string.");
                }}
                byte[] bytes = new byte[(int)byteLength];
                if (bytes.Length > 0)
                {{
                    Marshal.Copy(SlicePointer(slice), bytes, 0, bytes.Length);
                }}
                return Encoding.UTF8.GetString(bytes);
            }}
            finally
            {{
                SliceDrop(slice);
            }}
        }}
        finally
        {{
            if (addedReference) instance.DangerousRelease();
        }}
    }}
"#
                ));
            } else {
                field_imports.push(format!(
                    r#"    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    internal static extern {} {method_name}({struct_name} value);
"#,
                    getter.extern_fn_name, field_type.native
                ));
            }
            let expression = match field_type.conversion {
                CSharpConversion::Bool => {
                    format!("{struct_name}Native.{method_name}(this) != 0")
                }
                CSharpConversion::String => {
                    format!("{struct_name}Native.{method_name}(this)")
                }
                CSharpConversion::Enum => {
                    format!(
                        "({}){struct_name}Native.{method_name}(this)",
                        field_type.managed
                    )
                }
                _ => format!("{struct_name}Native.{method_name}(this)"),
            };
            accessors.push(format!("get => {expression};"));
        }
        if let Some(setter) = &field.setter {
            let method_name = to_pascal_case(&setter.name.to_string());
            if is_string {
                has_string_field = true;
                let native_method = format!("{method_name}Native");
                field_imports.push(format!(
                    r#"    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    private static extern void {native_method}({struct_name} instance, IntPtr value, nuint length);
"#,
                    setter.extern_fn_name
                ));
                field_helpers.push(format!(
                    r#"    internal static void {method_name}({struct_name} instance, string value)
    {{
        if (value is null) throw new ArgumentNullException(nameof(value));
        if (value.IndexOf('\0') >= 0) throw new ArgumentException("String fields cannot contain NUL characters.", nameof(value));
        IntPtr utf8 = Marshal.StringToCoTaskMemUTF8(value);
        if (utf8 == IntPtr.Zero) throw new OutOfMemoryException();
        try
        {{
            {native_method}(instance, utf8, (nuint)Encoding.UTF8.GetByteCount(value));
        }}
        finally
        {{
            Marshal.FreeCoTaskMem(utf8);
        }}
    }}
"#
                ));
            } else {
                field_imports.push(format!(
                    r#"    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    internal static extern void {method_name}({struct_name} instance, {} value);
"#,
                    setter.extern_fn_name, field_type.native
                ));
            }
            let value = match field_type.conversion {
                CSharpConversion::Bool => "value ? (byte)1 : (byte)0",
                CSharpConversion::Enum => "(int)value",
                _ => "value",
            };
            let call = if is_string {
                format!("{struct_name}Native.{method_name}(this, value)")
            } else {
                format!("{struct_name}Native.{method_name}(this, {value})")
            };
            accessors.push(format!("set => {call};"));
        }
        if !accessors.is_empty() {
            properties.push(format!(
                "    public {} {field_name} {{ {} }}",
                field_type.managed,
                accessors.join(" ")
            ));
        }
    }
    if has_string_getter {
        field_imports.push(format!(
            r#"    [DllImport("{library_name}", EntryPoint = "hiFfi__slice_ptr", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr SlicePointer(IntPtr slice);

    [DllImport("{library_name}", EntryPoint = "hiFfi__slice_len", CallingConvention = CallingConvention.Cdecl)]
    private static extern nuint SliceLength(IntPtr slice);

    [DllImport("{library_name}", EntryPoint = "hiFfi__slice_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern void SliceDrop(IntPtr slice);
"#
        ));
    }
    let field_imports = field_imports.join("\n");
    let field_helpers = field_helpers.join("\n");
    let properties = properties.join("\n\n");
    let default_constructor = struct_wrapper
        .default_constructor
        .as_ref()
        .map(|constructor| {
            format!(
                r#"
    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr CreateDefault();
"#,
                constructor.extern_fn_name
            )
        })
        .unwrap_or_default();
    let public_constructor = struct_wrapper
        .default_constructor
        .as_ref()
        .map(|_| {
            format!(
                r#"
    public {struct_name}() : base(IntPtr.Zero, true)
    {{
        SetHandle({struct_name}Native.CreateDefault());
        if (IsInvalid)
        {{
            throw new InvalidOperationException("Rust returned a null struct pointer.");
        }}
    }}
"#
            )
        })
        .unwrap_or_default();
    let string_import = if has_string_field {
        "using System.Text;\n"
    } else {
        ""
    };
    let collection_import = if has_collection_field {
        "using System.Collections.Generic;\n"
    } else {
        ""
    };
    let source = format!(
        r#"using System;
using System.Runtime.InteropServices;
using static CSharpReusableHelpers;
{string_import}
{collection_import}
using Microsoft.Win32.SafeHandles;

internal static class {struct_name}Native
{{
    [DllImport("{library_name}", EntryPoint = "{}", CallingConvention = CallingConvention.Cdecl)]
    internal static extern void Drop(IntPtr value);
{default_constructor}
{field_imports}
{field_helpers}
}}

public sealed partial class {struct_name} : SafeHandle
{{
    internal {struct_name}(IntPtr value) : base(IntPtr.Zero, true)
    {{
        if (value == IntPtr.Zero)
        {{
            throw new InvalidOperationException("Rust returned a null struct pointer.");
        }}
        SetHandle(value);
    }}

    internal static {struct_name} FromNative(IntPtr value) => new(value);
    internal IntPtr TakeHandle()
    {{
        if (IsClosed || IsInvalid) throw new ObjectDisposedException(nameof({struct_name}));
        IntPtr value = handle;
        SetHandleAsInvalid();
        return value;
    }}
{public_constructor}
{properties}

    public override bool IsInvalid => handle == IntPtr.Zero;

    protected override bool ReleaseHandle()
    {{
        {struct_name}Native.Drop(handle);
        return true;
    }}
}}
"#,
        struct_wrapper.drop_ext_fn_name
    );
    let output_dir = c_sharp_output_dir();
    create_file(source, output_dir.join(format!("{struct_name}.cs")));
}

fn c_sharp_output_dir() -> std::path::PathBuf {
    let output_dir = Path::new(GEN_CODE_DIR).join(C_SHARP_CODE_DIR);
    C_SHARP_OUTPUT_INITIALIZED.call_once(|| {
        if output_dir.exists() {
            fs::remove_dir_all(&output_dir).expect("Unable to clean C# generated-code directory");
        }
        fs::create_dir_all(&output_dir).expect("Unable to create C# generated-code directory");
    });
    output_dir
}

fn to_pascal_case(function_name: &str) -> String {
    function_name
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
