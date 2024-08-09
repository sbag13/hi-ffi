use crate::wrapper::*;
use quote::ToTokens;

pub fn gen_function_header(function_wrapper: &FunctionWrapper) -> String {
    let extern_fn_name = &function_wrapper.extern_function_name;
    let swift_args = function_wrapper
        .args_wrappers
        .iter()
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
        })
        .collect::<Vec<_>>();

    let swift_args = swift_args.join(", ");

    let ReturnTypes {
        cpp_return_type, ..
    } = map_return_type(&function_wrapper.return_wrapper);

    format!(r#"{cpp_return_type} {extern_fn_name}({swift_args});"#)
}

pub fn gen_function_definition(function_wrapper: &FunctionWrapper) -> String {
    let fn_name = function_wrapper.name.to_string();
    let extern_fn_name = &function_wrapper.extern_function_name;
    let (mut args_signatures, mut args_names, mut args_casts): (Vec<_>, Vec<_>, Vec<_>) =
        (Vec::new(), Vec::new(), Vec::new());
    function_wrapper
        .args_wrappers
        .iter()
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
        });
    let args_signatures = args_signatures.join(", ");
    let args_names = args_names.join(", ");
    let args_casts = args_casts.join("\n");

    let ReturnTypes {
        return_type_sig,
        result_cast,
        ..
    } = map_return_type(&function_wrapper.return_wrapper);

    let fn_definition = match (return_type_sig, result_cast) {
        (Some(return_type_sig), Some(result_cast)) => {
            format!(
                r#"
public func {fn_name}({args_signatures}) {return_type_sig}{{
{args_casts}
    let result = {extern_fn_name}({args_names})
{result_cast}
    return casted_result
}}"#
            )
        }
        (Some(return_type_sig), None) => {
            format!(
                r#"
public func {fn_name}({args_signatures}) {return_type_sig}{{
{args_casts}
    return {extern_fn_name}({args_names})
}}"#
            )
        }
        (None, _) => {
            format!(
                r#"
public func {fn_name}({args_signatures}) {{
{args_casts}
    {extern_fn_name}({args_names})
}}"#
            )
        }
    };

    format!(
        r#"@_exported import CFfiModule
{fn_definition}"#
    )
}

struct ReturnTypes {
    return_type_sig: Option<String>,
    cpp_return_type: String,
    result_cast: Option<String>,
}

fn map_return_type(return_wrapper: &Option<FunctionReturnWrapper>) -> ReturnTypes {
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
        None => ReturnTypes {
            return_type_sig: None,
            cpp_return_type: "void".to_string(),
            result_cast: None,
        },
    }
}
