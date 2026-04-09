use std::fmt::Display;

use crate::prepend_each_line_with_n_tabs;
use crate::wrapper::*;
use quote::ToTokens;

pub fn map_header_declaration_args(args: &[FunctionArgWrapper]) -> String {
    args.iter()
        .map(|arg| match arg {
            FunctionArgWrapper {
                arg_name,
                arg_type,
                wrapper_type:
                    WrapperType::IntegerNumber(_)
                    | WrapperType::Bool
                    | WrapperType::FloatingPointNumber(_),
            } => {
                format!("{} {}", arg_type.to_token_stream(), arg_name)
            }
            FunctionArgWrapper {
                arg_name,
                wrapper_type:
                    WrapperType::Struct(_)
                    | WrapperType::String
                    | WrapperType::Vec(_)
                    | WrapperType::Result(_)
                    | WrapperType::Option(_),
                ..
            } => {
                format!("void* {arg_name}")
            }

            FunctionArgWrapper {
                wrapper_type: WrapperType::Enum(_),
                arg_name,
                arg_type,
            } => {
                format!(
                    "enum {arg_type} {arg_name}",
                    arg_type = arg_type.to_token_stream()
                )
            }

            FunctionArgWrapper {
                wrapper_type: WrapperType::Trait(_),
                arg_name,
                ..
            } => {
                format!("struct RustTraitBridge {arg_name}")
            }

            FunctionArgWrapper {
                wrapper_type: WrapperType::UnitExpr,
                ..
            } => panic!("Unsupported type for function argument (empty expression)"),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn gen_function_header(
    extern_fn_name: &impl Display,
    args: &[FunctionArgWrapper],
    return_wrapper: &Option<FunctionReturnWrapper>,
) -> String {
    let c_args = map_header_declaration_args(args);

    let ReturnTypes {
        cpp_return_type, ..
    } = map_return_type(return_wrapper);

    format!(r#"{cpp_return_type} {extern_fn_name}({c_args});"#)
}

pub struct MappedSwiftFunctionArgsTokens {
    pub args_signatures: String,
    pub args_names: String,
    pub args_casts: String,
}

pub fn map_args<'a>(
    args: impl Iterator<Item = &'a FunctionArgWrapper>,
) -> MappedSwiftFunctionArgsTokens {
    let (mut args_signatures, mut args_names, mut args_casts): (Vec<_>, Vec<_>, Vec<_>) =
        (Vec::new(), Vec::new(), Vec::new());
    args
        .for_each(|arg| match arg {
            FunctionArgWrapper {
                wrapper_type: WrapperType::Trait(trait_name),
                arg_name,
                ..
            } => {
                args_signatures.push(format!("_ {arg_name}: {trait_name}"));
                args_casts.push(format!(r#"
let obj = Unmanaged.passRetained({arg_name} as AnyObject).toOpaque()

let bridge = {trait_name}Bridge(
    obj: obj,
    vtable: &global{trait_name}VTable,
    deleter: swift_{trait_name}_deleter
)
"#));
                args_names.push("bridge".to_string());
            },

            FunctionArgWrapper {
                arg_name,
                arg_type,
                wrapper_type: WrapperType::IntegerNumber(_) | WrapperType::Bool | WrapperType::FloatingPointNumber(_),
            } => {
                args_signatures.push(format!("_ {}: {}", arg_name, arg_type.to_token_stream()));
                args_names.push(arg_name.to_string());
            }

            FunctionArgWrapper {
                arg_name,
                wrapper_type: WrapperType::String,
                ..
            } => {
                args_signatures.push(format!("_ {arg_name}: String"));
                args_names.push(format!("casted_{arg_name}"));
                args_casts.push(format!(
                    r#"let casted_{arg_name} = {arg_name}.utf8CString.withUnsafeBufferPointer({{ ptr in return UnsafeMutableRawPointer(mutating: ptr.baseAddress!) }})"#
                ));
            }

            FunctionArgWrapper {
                arg_name,
                wrapper_type: WrapperType::Struct(_),
                arg_type,
            } => {
                args_signatures.push(format!("_ {arg_name}: {}", arg_type.to_token_stream()));
                args_names.push(format!("casted_{arg_name}"));
                args_casts.push(format!(
                    r#"let casted_{arg_name} = {arg_name}.rawPtr()"#
                ));
            }

            FunctionArgWrapper {
                arg_name,
                wrapper_type: WrapperType::Vec(inner_type),
                ..
            } => {
                let swift_type = match &**inner_type {
                    WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) | WrapperType::Bool => {
                        format!("[{}]", inner_type.name())
                    }
                    WrapperType::String => "[String]".to_string(),
                    WrapperType::Struct(name) => format!("[{}]", name),
                    WrapperType::Vec(_) => panic!("Vec of vecs not supported"),
                    WrapperType::Result(_) => panic!("Vec of results not supported"),
                    WrapperType::Option(_) => panic!("Vec of options not supported"),
                    WrapperType::Enum(name) => format!("[{}]", name),
                    WrapperType::UnitExpr => panic!("Empty expression is not supported as vec inner type"),
                    WrapperType::Trait(_) => panic!("Trait not supported as vec inner type"),
                };
                args_signatures.push(format!("_ {arg_name}: {swift_type}"));
                args_names.push(format!("casted_{arg_name}.rawPtr()"));
                args_casts.push(format!(
                    r#"let casted_{arg_name} = Rust{}Vec.fromSwift({arg_name})"#,
                    inner_type.name()
                ));
            }

            FunctionArgWrapper {
                arg_name,
                wrapper_type: WrapperType::Enum(_),
                arg_type,
            } => {
                args_signatures.push(format!("_ {arg_name}: {}", arg_type.to_token_stream()));
                args_casts.push(format!("let casted_{arg_name} = CFfiModule.{arg_type}(rawValue: UInt32({arg_name}.rawValue))", arg_type = arg_type.to_token_stream())); // Convert to C enum type with UInt32
                args_names.push(format!("casted_{arg_name}"));
            },

            FunctionArgWrapper {
                arg_name,
                wrapper_type: WrapperType::Option(inner_type),
                ..
            } => {
                let swift_inner_type = match &**inner_type {
                    WrapperType::IntegerNumber(t) | WrapperType::FloatingPointNumber(t) => t.to_string(),
                    WrapperType::Bool => "Bool".to_string(),
                    WrapperType::String => "String".to_string(),
                    WrapperType::Struct(name) => name.to_string(),
                    WrapperType::Vec(vec_inner) => format!("[{}]", vec_inner.name()),
                    WrapperType::Enum(name) => name.to_string(),
                    WrapperType::Result(_) => panic!("Result in option not supported"),
                    WrapperType::Option(_) => panic!("Option of options not supported"),
                    WrapperType::UnitExpr => "Void".to_string(),
                    WrapperType::Trait(_) => panic!("Trait not supported in option"),
                };
                let wrapper_name = format!("Rust{}Option", inner_type.name());
                args_signatures.push(format!("_ {arg_name}: {swift_inner_type}?"));
                args_names.push(format!("casted_{arg_name}.rawPtr()"));
                args_casts.push(format!(
                    r#"let casted_{arg_name} = {wrapper_name}.fromSwift({arg_name})"#
                ));
            }

            FunctionArgWrapper {
                wrapper_type: WrapperType::Result(_),
                ..
            } => panic!("Result type not supported as function argument"),

             FunctionArgWrapper {
                wrapper_type: WrapperType::UnitExpr,
                ..
            } => panic!("Unsupported type for function argument (empty expression)"),

            // No other variants
        });
    let args_signatures = args_signatures.join(", ");
    let args_names = args_names.join(", ");
    let args_casts = args_casts.join("\n");

    MappedSwiftFunctionArgsTokens {
        args_signatures,
        args_names,
        args_casts,
    }
}

pub fn compose_function_definition(
    fn_name: impl Display,
    extern_fn_name: impl Display,
    args: MappedSwiftFunctionArgsTokens,
    return_types: ReturnTypes,
    is_static: bool,
) -> String {
    let static_keyword = if is_static { "static " } else { "" };
    let return_type_sig = return_types.return_type_sig;
    let result_cast = return_types.result_cast;
    let MappedSwiftFunctionArgsTokens {
        args_signatures,
        args_names,
        args_casts,
    } = args;

    let (body, return_type) = match (return_type_sig.as_ref(), result_cast) {
        (Some(return_type), Some(result_cast)) => {
            let body = format!(
                "let result = {extern_fn_name}({args_names})
{result_cast}
return casted_result"
            );
            (body, return_type.as_str())
        }
        (Some(return_type), None) => {
            let body = format!("return {extern_fn_name}({args_names})");
            (body, return_type.as_str())
        }
        (None, _) => {
            let body = format!("{extern_fn_name}({args_names})");
            (body, "")
        }
    };

    let args_casts = prepend_each_line_with_n_tabs(&args_casts, 1);
    let body = prepend_each_line_with_n_tabs(&body, 1);

    format!(
        r#"
public {static_keyword}func {fn_name}({args_signatures}){return_type} {{
{args_casts}
{body}
}}"#
    )
}

pub fn gen_function_definition(function: &FunctionWrapper) -> String {
    let mapped_args = map_args(function.args.iter());
    let return_types = map_return_type(&function.return_wrapper);

    compose_function_definition(
        &function.name,
        &function.extern_function_name,
        mapped_args,
        return_types,
        false,
    )
}

pub struct ReturnTypes {
    pub return_type_sig: Option<String>,
    pub cpp_return_type: String,
    pub result_cast: Option<String>,
}

pub fn map_return_type(return_wrapper: &Option<FunctionReturnWrapper>) -> ReturnTypes {
    match return_wrapper {
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Trait(trait_name),
            ..
        }) => ReturnTypes {
            return_type_sig: Some(format!(" -> {}", trait_name)),
            cpp_return_type: "void*".to_string(),
            result_cast: Some(format!("let casted_result = {}Impl(result!)", trait_name)),
        },

        Some(FunctionReturnWrapper {
            wrapper_type:
                WrapperType::IntegerNumber(_) | WrapperType::Bool | WrapperType::FloatingPointNumber(_),
            return_type,
        }) => ReturnTypes {
            return_type_sig: Some(format!(" -> {}", return_type.to_token_stream())),
            cpp_return_type: format!("{}", return_type.to_token_stream()),
            result_cast: None,
        },

        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::String,
            ..
        }) => ReturnTypes {
            return_type_sig: Some(" -> String".to_string()),
            cpp_return_type: "void*".to_string(),
            result_cast: Some("let casted_result = RustString(result!).to_string()".to_string()),
        },

        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Struct(_),
            return_type,
        }) => ReturnTypes {
            return_type_sig: Some(format!(" -> {}", return_type.to_token_stream())),
            cpp_return_type: "void*".to_string(),
            result_cast: Some(format!(
                "let casted_result = {}(result!)",
                return_type.to_token_stream()
            )),
        },

        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Vec(inner_type),
            ..
        }) => {
            let swift_type = match &**inner_type {
                WrapperType::IntegerNumber(_)
                | WrapperType::FloatingPointNumber(_)
                | WrapperType::Bool => {
                    format!("[{}]", inner_type.name())
                }
                WrapperType::String => "[String]".to_string(),
                WrapperType::Struct(name) => format!("[{}]", name),
                WrapperType::Vec(_) => panic!("Vec of vecs not supported"),
                WrapperType::Result(_) => panic!("Vec of results not supported"),
                WrapperType::Option(_) => panic!("Vec of options not supported"),
                WrapperType::Enum(name) => format!("[{}]", name),
                WrapperType::UnitExpr => {
                    panic!("Empty expression is not supported as vec inner type")
                }
                WrapperType::Trait(_) => panic!("Trait not supported as vec inner type"),
            };
            ReturnTypes {
                return_type_sig: Some(format!(" -> {}", swift_type)),
                cpp_return_type: "void*".to_string(),
                result_cast: Some(format!(
                    "let casted_result = Rust{}Vec(result!).toSwift()",
                    inner_type.name()
                )),
            }
        }

        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Enum(_),
            return_type,
        }) => ReturnTypes {
            return_type_sig: Some(format!(" -> {}", return_type.to_token_stream())),
            cpp_return_type: format!("enum {}", return_type.to_token_stream()),
            result_cast: Some(format!(
                "let casted_result = {}(rawValue: Int32(result.rawValue))!",
                return_type.to_token_stream()
            )),
        },

        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Result(inner),
            ..
        }) => {
            let inner_name = inner.name();
            let swift_type = match &**inner {
                WrapperType::IntegerNumber(_)
                | WrapperType::FloatingPointNumber(_)
                | WrapperType::Bool => inner.name().to_string(),
                WrapperType::String => "String".to_string(),
                WrapperType::Struct(name) => name.to_string(),
                WrapperType::Vec(vec_inner) => format!("[{}]", vec_inner.name()),
                WrapperType::Enum(name) => name.to_string(),
                WrapperType::Result(_) => panic!("Nested Results not supported"),
                WrapperType::Option(_) => panic!("Option in result not supported"),
                WrapperType::UnitExpr => "Void".to_string(),
                WrapperType::Trait(_) => panic!("Trait not supported in result"),
            };
            ReturnTypes {
                return_type_sig: Some(format!(" throws -> {swift_type}")),
                cpp_return_type: match &**inner {
                    WrapperType::Result(_) => panic!("Nested Results not supported"),
                    _ => "void*".to_string(),
                },
                result_cast: Some(format!(
                    "
let wrapped_rust_result = Rust{inner_name}Result(result!)
let casted_result: {swift_type}
if wrapped_rust_result.isErr() {{
    throw RustError(wrapped_rust_result.unwrapErr())
}} else {{
    casted_result = wrapped_rust_result.unwrap()
}}
"
                )),
            }
        }

        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Option(inner),
            ..
        }) => {
            let inner_name = inner.name();
            let wrapper_name = format!("Rust{}Option", inner_name);
            let swift_type = match &**inner {
                WrapperType::IntegerNumber(_)
                | WrapperType::FloatingPointNumber(_)
                | WrapperType::Bool => inner.name().to_string(),
                WrapperType::String => "String".to_string(),
                WrapperType::Struct(name) => name.to_string(),
                WrapperType::Vec(vec_inner) => format!("[{}]", vec_inner.name()),
                WrapperType::Enum(name) => name.to_string(),
                WrapperType::Result(_) => panic!("Result in option not supported"),
                WrapperType::Option(_) => panic!("Option of options not supported"),
                WrapperType::UnitExpr => "Void".to_string(),
                WrapperType::Trait(_) => panic!("Trait not supported in option"),
            };
            ReturnTypes {
                return_type_sig: Some(format!(" -> {swift_type}?")),
                cpp_return_type: "void*".to_string(),
                result_cast: Some(format!(
                    "let casted_result = {wrapper_name}(result!).toSwift()"
                )),
            }
        }

        None
        | Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::UnitExpr,
            ..
        }) => ReturnTypes {
            return_type_sig: None,
            cpp_return_type: "void".to_string(),
            result_cast: None,
        },
    }
}
