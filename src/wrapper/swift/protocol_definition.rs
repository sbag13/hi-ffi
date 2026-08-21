use crate::wrapper::base::RUST_STRING_FROM_C_PTR_FN_NAME;
use crate::wrapper::{FunctionReturnWrapper, FunctionWrapper};
use std::fmt::Display;
use std::ops::Deref;

use crate::EXPORTED_SYMBOLS_PREFIX;
use crate::prepend_each_line_with_n_tabs;
use crate::wrapper::WrapperType;
use crate::wrapper::swift::function_definition::{
    MappedSwiftFunctionArgsTokens, ReturnTypes, map_args, map_return_type,
};
use crate::wrapper::trait_wrapper::TraitWrapper;

pub(crate) fn gen_trait_bridge_header(trait_wrapper: &TraitWrapper) -> String {
    let mut vtable_functions = trait_wrapper
        .functions
        .iter()
        .map(|method| {
            let ext_method_name = &method.extern_function_name;

            let wrapper_args = method
                .args
                .iter()
                .map(|arg| {
                    let ty = match &arg.wrapper_type {
                        WrapperType::IntegerNumber(_)
                        | WrapperType::FloatingPointNumber(_)
                        | WrapperType::Bool => arg.wrapper_type.name(),
                        WrapperType::Enum(_) => "i32".to_string(),
                        _ => "void*".to_string(),
                    };
                    let arg_name = &arg.arg_name;
                    format!(", {ty} {arg_name}")
                })
                .collect::<String>();

            let return_type = match &method.return_wrapper {
                Some(ret) => match &ret.wrapper_type {
                    WrapperType::IntegerNumber(ty) | WrapperType::FloatingPointNumber(ty) => {
                        ty.to_string()
                    }
                    WrapperType::Enum(_) => "i32".to_string(),
                    WrapperType::Bool => "bool".to_string(),
                    WrapperType::String
                    | WrapperType::Struct(_)
                    | WrapperType::Vec(_)
                    | WrapperType::Option(_)
                    | WrapperType::Result(_) => "void*".to_string(),
                    WrapperType::UnitExpr => "void".to_string(),
                    WrapperType::Trait(_) => {
                        panic!("Trait return type is not supported in traits")
                    }
                },
                None => "void".to_string(),
            };

            format!("    {return_type} (*{ext_method_name})(void* self{wrapper_args});\n",)
        })
        .collect::<String>();
    vtable_functions.pop(); // remove last comma and newline

    let class_name = &trait_wrapper.name;

    // Generate BoxDyn extern function declarations
    let mut boxdyn_externs = String::new();
    for method in &trait_wrapper.functions {
        let boxdyn_name = format!("{}_BoxDyn", method.extern_function_name);

        let wrapper_args = method
            .args
            .iter()
            .map(|arg| {
                let ty = match &arg.wrapper_type {
                    WrapperType::IntegerNumber(_)
                    | WrapperType::FloatingPointNumber(_)
                    | WrapperType::Bool => arg.wrapper_type.name(),
                    WrapperType::Enum(_) => "i32".to_string(),
                    _ => "void*".to_string(),
                };
                let arg_name = &arg.arg_name;
                format!(", {ty} {arg_name}")
            })
            .collect::<String>();

        let return_type = match &method.return_wrapper {
            Some(ret) => match &ret.wrapper_type {
                WrapperType::IntegerNumber(ty) | WrapperType::FloatingPointNumber(ty) => {
                    ty.to_string()
                }
                WrapperType::Enum(_) => "i32".to_string(),
                WrapperType::Bool => "bool".to_string(),
                WrapperType::String
                | WrapperType::Struct(_)
                | WrapperType::Vec(_)
                | WrapperType::Option(_)
                | WrapperType::Result(_) => "void*".to_string(),
                WrapperType::UnitExpr => "void".to_string(),
                WrapperType::Trait(_) => {
                    panic!("Trait return type is not supported in traits")
                }
            },
            None => "void".to_string(),
        };

        boxdyn_externs.push_str(&format!(
            "{return_type} {boxdyn_name}(void* self{wrapper_args});\n"
        ));
    }

    let drop_fn = format!("void {EXPORTED_SYMBOLS_PREFIX}{class_name}_BoxDyn_drop(void* self);\n");

    format!(
        r#"typedef struct {class_name}VTable {class_name}VTable;
typedef struct {class_name}Bridge {class_name}Bridge;

struct {class_name}VTable {{
{vtable_functions}
}};

struct {class_name}Bridge {{
    void* obj;
    struct {class_name}VTable* vtable;
    void (*deleter)(void*);
}};

// BoxDyn wrapper function pointers
{boxdyn_externs}{drop_fn}"#
    )
}

pub(crate) fn gen_protocol_definition(trait_wrapper: &TraitWrapper) -> String {
    // Generate method signatures for each method in the trait
    let functions = trait_wrapper
        .functions
        .iter()
        .map(gen_method_signature)
        .collect::<Vec<_>>();
    let functions = functions.join("\n");

    let vtable_functions = trait_wrapper
        .functions
        .iter()
        .map(|f| gen_vtable_function(f, &trait_wrapper.name))
        .collect::<Vec<String>>();
    let vtable_functions = vtable_functions.join("\n");

    let trait_name = &trait_wrapper.name;

    let global_vtable_fields = trait_wrapper
        .functions
        .iter()
        .map(|f| {
            let ext_fn_name = &f.extern_function_name;
            format!("    {ext_fn_name}: {ext_fn_name}")
        })
        .collect::<Vec<String>>()
        .join(",\n");

    let deleter = format!(
        r#"
func swift_{trait_name}_deleter(data: UnsafeMutableRawPointer?) {{
    Unmanaged<AnyObject>.fromOpaque(data!).release()
}}
"#
    );

    // Generate the trait impl class for BoxDyn
    let trait_impl_class = gen_trait_box_dyn_impl_class(trait_wrapper);

    format!(
        r#"{vtable_functions}
{deleter}

var global{trait_name}VTable = {trait_name}VTable (
{global_vtable_fields}
)

public protocol {trait_name} {{
{functions}
}}

{trait_impl_class}"#
    )
}

fn gen_method_signature(function_wrapper: &FunctionWrapper) -> String {
    let fn_name = &function_wrapper.name;

    let MappedSwiftFunctionArgsTokens {
        args_signatures, ..
    } = map_args(function_wrapper.args.iter());

    let ReturnTypes {
        return_type_sig, ..
    } = map_return_type(&function_wrapper.return_wrapper);
    let return_type_sig = return_type_sig.unwrap_or_default();

    format!("    func {fn_name}({args_signatures}){return_type_sig}")
}

fn vtable_function_args(function_wrapper: &FunctionWrapper) -> String {
    function_wrapper
        .args
        .iter()
        .map(|arg_wrapper| {
            let arg_name = &arg_wrapper.arg_name;
            let arg_type = vtable_arg_type(&arg_wrapper.wrapper_type);
            format!(", {arg_name}: {arg_type}")
        })
        .collect()
}

fn vtable_arg_type(arg_wrapper: &WrapperType) -> String {
    match arg_wrapper {
        WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) | WrapperType::Bool => {
            arg_wrapper.name()
        }
        WrapperType::Enum(_) => "Int32".to_string(),
        WrapperType::Struct(_)
        | WrapperType::Vec(_)
        | WrapperType::Option(_)
        | WrapperType::String => "UnsafeMutableRawPointer?".to_string(),
        WrapperType::Result(_) => panic!("Unsupported argument type in trait method"),
        WrapperType::UnitExpr => panic!("Unsupported argument type in trait method"),
        WrapperType::Trait(_) => panic!("Trait cannot be used as argument in trait method"),
    }
}

fn get_vtable_ret_sig(return_wrapper: &Option<FunctionReturnWrapper>) -> String {
    return_wrapper
        .as_ref()
        .map(|ret_wrapper| match ret_wrapper.wrapper_type {
            WrapperType::IntegerNumber(_)
            | WrapperType::FloatingPointNumber(_)
            | WrapperType::Bool => format!(" -> {}", ret_wrapper.wrapper_type.name()),
            WrapperType::Enum(_) => " -> Int32".to_string(),
            WrapperType::Struct(_)
            | WrapperType::Vec(_)
            | WrapperType::Option(_)
            | WrapperType::String => " -> UnsafeMutableRawPointer?".to_string(),
            WrapperType::Result(_) => " -> UnsafeMutableRawPointer?".to_string(),
            WrapperType::UnitExpr => " -> Void".to_string(),
            WrapperType::Trait(_) => panic!("Trait cannot be used as return type in trait method"),
        })
        .unwrap_or(" -> Void".to_string())
}

fn gen_vtable_function(function_wrapper: &FunctionWrapper, trait_name: impl Display) -> String {
    let extern_fn_name = &function_wrapper.extern_function_name;

    let args_signatures = vtable_function_args(function_wrapper);

    let return_type_sig = get_vtable_ret_sig(&function_wrapper.return_wrapper);

    let fn_name = &function_wrapper.name;

    let (mut args_casts, mut args_names): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    function_wrapper
        .args
        .iter()
        .for_each(|arg_wrapper| match &arg_wrapper.wrapper_type {
            WrapperType::IntegerNumber(_)
            | WrapperType::FloatingPointNumber(_)
            | WrapperType::Bool => args_names.push(arg_wrapper.arg_name.to_string()),
            WrapperType::Enum(_) => {
                args_casts.push(format!(
                    "let casted_{} = FfiModule.{}(rawValue: Int32({}))!",
                    arg_wrapper.arg_name,
                    arg_wrapper.wrapper_type.name(),
                    arg_wrapper.arg_name
                ));
                args_names.push(format!("casted_{}", arg_wrapper.arg_name));
            }
            WrapperType::Struct(s_name) => {
                let arg_name = &arg_wrapper.arg_name;
                args_casts.push(format!("let casted_{arg_name} = {s_name}({arg_name}!)",));
                args_names.push(format!("casted_{}", arg_wrapper.arg_name));
            }
            WrapperType::String => {
                let arg_name = &arg_wrapper.arg_name;
                args_casts.push(format!(
                    "let casted_{arg_name} = RustString({arg_name}!).to_string()"
                ));
                args_names.push(format!("casted_{}", arg_wrapper.arg_name));
            }
            WrapperType::Vec(inner) => {
                let arg_name = &arg_wrapper.arg_name;
                let inner_name = inner.name();
                args_casts.push(format!(
                    "let casted_{arg_name} = Rust{inner_name}Vec({arg_name}!).toSwift()",
                ));
                args_names.push(format!("casted_{}", arg_wrapper.arg_name));
            }
            WrapperType::Option(inner) => {
                let arg_name = &arg_wrapper.arg_name;
                let inner_name = inner.name();
                args_casts.push(format!(
                    "let casted_{arg_name} = Rust{inner_name}Option({arg_name}!).toSwift()",
                ));
                args_names.push(format!("casted_{}", arg_wrapper.arg_name));
            }
            WrapperType::Result(_) => panic!("Unsupported argument type in trait method"),
            WrapperType::UnitExpr => panic!("Unsupported argument type in trait method"),
            WrapperType::Trait(_) => panic!("Trait cannot be used as argument in trait method"),
        });

    let args_casts = prepend_each_line_with_n_tabs(&args_casts.join("\n"), 1);
    let args_names = args_names.join(", ");

    let return_result = match &function_wrapper.return_wrapper {
        Some(ret_wrapper) => match &ret_wrapper.wrapper_type {
            WrapperType::IntegerNumber(_)
            | WrapperType::FloatingPointNumber(_)
            | WrapperType::Bool
            | WrapperType::UnitExpr => {
                format!("    return obj_ptr.{fn_name}({args_names})")
            }
            WrapperType::Enum(_) => {
                format!("    return obj_ptr.{fn_name}({args_names}).rawValue")
            }
            WrapperType::Struct(_) => format!("    return obj_ptr.{fn_name}({args_names}).leak()",),
            WrapperType::Vec(inner) => format!(
                "    return Rust{inner_name}Vec.fromSwift(obj_ptr.{fn_name}({args_names})).leak()",
                inner_name = inner.name()
            ),
            WrapperType::Option(inner) => format!(
                "    return Rust{inner_name}Option.fromSwift(obj_ptr.{fn_name}({args_names})).leak()",
                inner_name = inner.name()
            ),
            WrapperType::String => {
                format!(
                    "    return obj_ptr.{fn_name}({args_names}).withCString {{ cPtr in
        let rawPtr = UnsafeMutableRawPointer(mutating: cPtr)
        return CFfiModule.{RUST_STRING_FROM_C_PTR_FN_NAME}(rawPtr)
    }}"
                )
            }
            WrapperType::Result(inner) => {
                let inner_name = inner.name();

                let result_cast =  match inner.deref() {
                                WrapperType::String => "        let rust_ok_arg = swift_result.utf8CString.withUnsafeBufferPointer({ ptr in return UnsafeMutableRawPointer(mutating: ptr.baseAddress!) })".to_string(),
                                WrapperType::Struct(_) => "        let rust_ok_arg = swift_result.rawPtr()".to_string(),
                                WrapperType::Vec(inner) => format!("        let rust_vec = Rust{inner}Vec.fromSwift(swift_result)
        let rust_ok_arg = rust_vec.rawPtr()", inner = inner.name()),
                                WrapperType::Enum(inner) => format!("        let rust_ok_arg = CFfiModule.{inner}(rawValue: UInt32(swift_result.rawValue))"),
                                WrapperType::UnitExpr => "".to_string(),
                                _ => "        let rust_ok_arg = swift_result".to_string(),
                            };

                let rust_ok_arg = match inner.deref() {
                    WrapperType::UnitExpr => "".to_string(),
                    _ => "rust_ok_arg".to_string(),
                };

                let swift_result_var_def = match inner.deref() {
                    WrapperType::UnitExpr => "".to_string(),
                    _ => "let swift_result = ".to_string(),
                };

                format!(
                    r#"    do {{
        {swift_result_var_def}try obj_ptr.{fn_name}({args_names})
{result_cast}
        return CFfiModule.{EXPORTED_SYMBOLS_PREFIX}{inner_name}_str_error_result_ok({rust_ok_arg})
    }} catch let error as RustError {{
        let message: String = error.description()
        let casted_msg = message.utf8CString.withUnsafeBufferPointer({{ ptr in return UnsafeMutableRawPointer(mutating: ptr.baseAddress!) }})
        return CFfiModule.{EXPORTED_SYMBOLS_PREFIX}{inner_name}_str_error_result_err(casted_msg)
    }} catch {{
        let message: String = error.localizedDescription
        let casted_msg = message.utf8CString.withUnsafeBufferPointer({{ ptr in return UnsafeMutableRawPointer(mutating: ptr.baseAddress!) }})
        return CFfiModule.{EXPORTED_SYMBOLS_PREFIX}{inner_name}_str_error_result_err(casted_msg)
    }}"#
                )
            }
            WrapperType::Trait(_) => panic!("Trait cannot be used as return type in trait method"),
        },
        None => format!("    obj_ptr.{fn_name}({args_names})"),
    };

    format!(
        r#"
@_cdecl("{extern_fn_name}")
func {extern_fn_name}(obj: UnsafeMutableRawPointer?{args_signatures}){return_type_sig} {{
    let obj_ptr = Unmanaged<AnyObject>.fromOpaque(obj!).takeUnretainedValue() as! {trait_name}
{args_casts}
{return_result}
}}"#
    )
}

pub(crate) fn gen_trait_box_dyn_impl_class(trait_wrapper: &TraitWrapper) -> String {
    let trait_name = &trait_wrapper.name;
    let mut methods = String::new();

    for method in &trait_wrapper.functions {
        let fn_name = &method.name;
        let boxdyn_extern_name = format!("{}_BoxDyn", method.extern_function_name);

        let MappedSwiftFunctionArgsTokens {
            args_signatures, ..
        } = map_args(method.args.iter());

        let ReturnTypes {
            return_type_sig,
            result_cast,
            ..
        } = map_return_type(&method.return_wrapper);

        let return_type_sig = return_type_sig.unwrap_or_default();

        // Build argument list for BoxDyn call - convert types appropriately
        let mut call_args = vec!["self_ptr".to_string()];
        let mut pre_call_casts = String::new();

        for arg in &method.args {
            let arg_name = &arg.arg_name;
            match &arg.wrapper_type {
                WrapperType::Enum(_) => {
                    // Enums: pass raw i32 value directly
                    call_args.push(format!("i32({arg_name}.rawValue)"));
                }
                WrapperType::String => {
                    // Strings: convert to C string and pass pointer
                    if !pre_call_casts.is_empty() {
                        pre_call_casts.push('\n');
                    }
                    pre_call_casts.push_str(&format!(
                        "        let casted_{arg_name} = {arg_name}.utf8CString.withUnsafeBufferPointer({{ ptr in return UnsafeMutableRawPointer(mutating: ptr.baseAddress!) }})"
                    ));
                    call_args.push(format!("casted_{arg_name}"));
                }
                WrapperType::Struct(_) => {
                    // Structs: get raw pointer
                    if !pre_call_casts.is_empty() {
                        pre_call_casts.push('\n');
                    }
                    pre_call_casts.push_str(&format!(
                        "        let casted_{arg_name} = {arg_name}.rawPtr()"
                    ));
                    call_args.push(format!("casted_{arg_name}"));
                }
                WrapperType::Vec(inner_type) => {
                    // Vecs: convert to Rust vec wrapper
                    if !pre_call_casts.is_empty() {
                        pre_call_casts.push('\n');
                    }
                    pre_call_casts.push_str(&format!(
                        "        let casted_{arg_name} = Rust{}Vec.fromSwift({arg_name})",
                        inner_type.name()
                    ));
                    call_args.push(format!("casted_{arg_name}.rawPtr()"));
                }
                WrapperType::Option(inner_type) => {
                    // Options: convert using wrapper
                    if !pre_call_casts.is_empty() {
                        pre_call_casts.push('\n');
                    }
                    pre_call_casts.push_str(&format!(
                        "        let casted_{arg_name} = Rust{}Option.fromSwift({arg_name})",
                        inner_type.name()
                    ));
                    call_args.push(format!("casted_{arg_name}.rawPtr()"));
                }
                WrapperType::Trait(_)
                | WrapperType::IntegerNumber(_)
                | WrapperType::Bool
                | WrapperType::FloatingPointNumber(_) => {
                    // Primitive types: pass directly
                    call_args.push(arg_name.to_string());
                }
                WrapperType::Result(_) | WrapperType::UnitExpr => {
                    panic!("Unsupported type in trait method: {:?}", arg.wrapper_type);
                }
            }
        }

        let args_for_call = call_args.join(", ");
        let pre_call_str = if pre_call_casts.is_empty() {
            String::new()
        } else {
            format!("{}\n        ", pre_call_casts)
        };

        // For BoxDyn, we need to handle returns differently than regular functions
        // since BoxDyn returns primitives or void* directly
        let method_body = match &method.return_wrapper {
            Some(FunctionReturnWrapper {
                wrapper_type: WrapperType::Result(inner),
                ..
            }) => {
                // Result types: BoxDyn returns opaque result pointer, unwrap it
                let inner_name = inner.name();
                let result_class_name = format!("Rust{}Result", inner_name);
                format!(
                    "let result = CFfiModule.{boxdyn_extern_name}({args_for_call})\n        \
                    let wrapped_rust_result = {result_class_name}(result!)\n        \
                    if wrapped_rust_result.isErr() {{\n            \
                        throw RustError(wrapped_rust_result.unwrapErr())\n        \
                    }} else {{\n            \
                        return wrapped_rust_result.unwrap()\n        \
                    }}"
                )
            }
            Some(FunctionReturnWrapper {
                wrapper_type: WrapperType::Enum(enum_name),
                ..
            }) => {
                // Enum returns from BoxDyn are already i32, no need for result_cast
                format!(
                    "let result = CFfiModule.{boxdyn_extern_name}({args_for_call})\n        return {enum_name}(rawValue: Int32(result))!"
                )
            }
            Some(_) if result_cast.is_some() => {
                // Other complex return types (String, Struct, Vec, etc.)
                let cast = result_cast.unwrap();
                format!(
                    "let result = CFfiModule.{boxdyn_extern_name}({args_for_call})\n        {cast}\n        return casted_result"
                )
            }
            Some(_) => {
                // Primitive returns
                format!("return CFfiModule.{boxdyn_extern_name}({args_for_call})")
            }
            None => {
                // Void returns
                format!("CFfiModule.{boxdyn_extern_name}({args_for_call})")
            }
        };

        methods.push_str(&format!(
            r#"
    public func {fn_name}({args_signatures}){return_type_sig} {{
{pre_call_str}
        {method_body}
    }}
"#
        ));
    }

    format!(
        r#"public class {trait_name}Impl: {trait_name} {{
    private var self_ptr: UnsafeMutableRawPointer

    public init(_ ptr: UnsafeMutableRawPointer) {{
        self_ptr = ptr
    }}

    deinit {{
        CFfiModule.{EXPORTED_SYMBOLS_PREFIX}{trait_name}_BoxDyn_drop(self_ptr)
    }}{methods}
}}"#
    )
}
