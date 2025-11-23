use std::fmt::Display;

use crate::wrapper::*;
use quote::ToTokens;

pub fn map_header_declaration_args(args: &[FunctionArgWrapper]) -> String {
    args.iter()
        .map(|arg| match arg {
            FunctionArgWrapper {
                arg_name,
                arg_type,
                wrapper_type: FunctionArgWrapperType::Primitive,
            } => {
                format!("{} {}", arg_type.to_token_stream(), arg_name)
            }
            FunctionArgWrapper {
                arg_name,
                wrapper_type: FunctionArgWrapperType::String,
                ..
            } => {
                format!("void* {arg_name}")
            }
            FunctionArgWrapper { .. } => {
                panic!("Struct arguments are not supported");
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn gen_function_header(
    extern_fn_name: &impl Display,
    args: &[FunctionArgWrapper],
    return_wrapper: &Option<FunctionReturnWrapper>,
) -> String {
    let swift_args = map_header_declaration_args(args);

    let ReturnTypes {
        cpp_return_type, ..
    } = map_return_type(return_wrapper);

    format!(r#"{cpp_return_type} {extern_fn_name}({swift_args});"#)
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
                arg_name,
                arg_type,
                wrapper_type: FunctionArgWrapperType::Primitive,
            } => {
                args_signatures.push(format!("_ {}: {}", arg_name, arg_type.to_token_stream()));
                args_names.push(arg_name.to_string());
            }

            FunctionArgWrapper {
                arg_name,
                wrapper_type: FunctionArgWrapperType::String,
                ..
            } => {
                args_signatures.push(format!("_ {arg_name}: String"));
                args_names.push(format!("casted_{arg_name}"));
                args_casts.push(format!(
                    r#"    let casted_{arg_name} = {arg_name}.utf8CString.withUnsafeBufferPointer({{ ptr in return UnsafeMutableRawPointer(mutating: ptr.baseAddress!) }})"#
                ));
            }

            FunctionArgWrapper {
                ..
            } => {
                panic!("Struct arguments are not supported");
            }
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
                "    let result = {extern_fn_name}({args_names})
    {result_cast}
    return casted_result"
            );
            (body, return_type.as_str())
        }
        (Some(return_type), None) => {
            let body = format!("    return {extern_fn_name}({args_names})");
            (body, return_type.as_str())
        }
        (None, _) => {
            let body = format!("    {extern_fn_name}({args_names})");
            (body, "")
        }
    };

    format!(
        r#"
    public {static_keyword}func {fn_name}({args_signatures}) {return_type} {{
    {args_casts}
    {body}
    }}"#
    )
}

pub fn gen_function_definition(function: &FunctionWrapper) -> String {
    let mapped_args = map_args(function.args_wrappers.iter());
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
            wrapper_type: FunctionReturnWrapperType::Primitive,
            return_type,
        }) => ReturnTypes {
            return_type_sig: Some(format!("-> {} ", return_type.to_token_stream())),
            cpp_return_type: format!("{}", return_type.to_token_stream()),
            result_cast: None,
        },

        Some(FunctionReturnWrapper {
            wrapper_type: FunctionReturnWrapperType::String,
            ..
        }) => ReturnTypes {
            return_type_sig: Some("-> String ".to_string()),
            cpp_return_type: "void*".to_string(),
            result_cast: Some(
                "    let casted_result = RustString(result!).to_string()".to_string(),
            ),
        },

        Some(FunctionReturnWrapper { .. }) => {
            panic!("Struct return types are not supported");
        }

        None => ReturnTypes {
            return_type_sig: None,
            cpp_return_type: "void".to_string(),
            result_cast: None,
        },
    }
}
