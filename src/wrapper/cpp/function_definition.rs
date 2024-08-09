use crate::wrapper::cpp::*;
use quote::ToTokens;

pub fn gen_function_declaration(function_wrapper: &FunctionWrapper) -> String {
    let fn_name = function_wrapper.name.to_string();
    let extern_fn_name = &function_wrapper.extern_function_name;
    let (mut cpp_args, mut wrapper_args) = (Vec::new(), Vec::new());
    function_wrapper
        .args_wrappers
        .iter()
        .for_each(|arg| match arg {
            FunctionArgWrapper {
                arg_name,
                arg_type,
                wrapper_type: FunctionArgWrapperType::Primitive,
            } => {
                cpp_args.push(format!("{} {}", arg_type.to_token_stream(), arg_name));
                wrapper_args.push(format!("{} {}", arg_type.to_token_stream(), arg_name));
            }
            FunctionArgWrapper {
                arg_name,
                wrapper_type: FunctionArgWrapperType::String,
                ..
            } => {
                cpp_args.push(format!("std::string&& {arg_name}"));
                wrapper_args.push(format!("char* {arg_name}"));
            }
        });

    let cpp_args = cpp_args.join(", ");
    let wrapper_args = wrapper_args.join(", ");

    let ReturnTypes {
        ext_return_type,
        return_type,
        ..
    } = map_return_type(&function_wrapper.return_wrapper);

    format!(
        r#"
#include "base.h"

extern "C" {{
    {ext_return_type} {extern_fn_name}({wrapper_args});
}}

{return_type} {fn_name}({cpp_args});
"#
    )
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
                args_signatures.push(format!("{} {}", arg_type.to_token_stream(), arg_name));
                args_names.push(arg_name.to_string());
            }

            FunctionArgWrapper {
                arg_name,
                wrapper_type: FunctionArgWrapperType::String,
                ..
            } => {
                args_signatures.push(format!("std::string&& {arg_name}"));
                args_names.push(format!("casted_{arg_name}"));
                args_casts.push(format!(
                    r#"    auto casted_{arg_name} = {arg_name}.data();"#
                ));
            }
        });
    let args_signatures = args_signatures.join(", ");
    let args_names = args_names.join(", ");
    let args_casts = args_casts.join("\n");

    let ReturnTypes {
        return_type,
        return_cast,
        ext_return_type,
    } = map_return_type(&function_wrapper.return_wrapper);

    format!(
        r#"
#include "{fn_name}.h"

{return_type} {fn_name}({args_signatures}) {{
{args_casts}
    {ext_return_type} result = {extern_fn_name}({args_names});
{return_cast}
}}
"#
    )
}

struct ReturnTypes {
    ext_return_type: String,
    return_type: String,
    return_cast: String,
}

fn map_return_type(return_wrapper: &Option<FunctionReturnWrapper>) -> ReturnTypes {
    match return_wrapper {
        Some(FunctionReturnWrapper {
            wrapper_type: FunctionReturnWrapperType::Primitive,
            return_type,
        }) => ReturnTypes {
            ext_return_type: return_type.to_token_stream().to_string(),
            return_type: return_type.to_token_stream().to_string(),
            return_cast: "    return result;".to_string(),
        },
        Some(FunctionReturnWrapper {
            wrapper_type: FunctionReturnWrapperType::String,
            ..
        }) => ReturnTypes {
            ext_return_type: "void*".to_string(),
            return_type: "std::string".to_string(),
            return_cast: "
    auto rust_str = RustString(result);
    return rust_str.to_string();"
                .to_string(),
        },
        None => ReturnTypes {
            ext_return_type: "void*".to_string(),
            return_type: "void".to_string(),
            return_cast: "".to_string(),
        },
    }
}
