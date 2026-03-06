use crate::wrapper::base::RUST_STRING_FROM_C_PTR_FN_NAME;
use crate::wrapper::{FunctionReturnWrapper, FunctionWrapper};
use std::fmt::Display;

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

    format!(
        r#"struct {class_name}VTable {{
{vtable_functions}
}};

struct {class_name}Bridge {{
    void* obj;
    struct {class_name}VTable* vtable;
    void (*deleter)(void*);
}};
"#
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

    format!(
        r#"{vtable_functions}
{deleter}

var global{trait_name}VTable = {trait_name}VTable (
{global_vtable_fields}
)

public protocol {trait_name} {{
{functions}
}}"#
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
            WrapperType::Result(_) => panic!("Unsupported return type in trait method"),
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
            WrapperType::Result(_) => panic!("Unsupported return type in trait method"),

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
