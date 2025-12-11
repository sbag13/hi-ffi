use std::collections::HashSet;

use crate::{prepend_each_line_with_n_tabs, wrapper::cpp::*};
use quote::ToTokens;

pub struct MappedCppFunctionArgsTokens {
    pub cpp_args: String,
    pub wrapper_args: String,
    pub arg_names: String,
    pub arg_casts: String,
    pub includes: HashSet<String>,
}
pub fn map_args<'a>(
    args: impl Iterator<Item = &'a FunctionArgWrapper>,
) -> MappedCppFunctionArgsTokens {
    let (mut cpp_args, mut wrapper_args, mut arg_names, mut arg_casts, mut includes): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        HashSet<String>,
    ) = (
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        HashSet::new(),
    );
    args.for_each(|arg| match arg {
        FunctionArgWrapper {
            arg_name,
            arg_type,
            wrapper_type: FunctionArgWrapperType::Primitive,
        } => {
            cpp_args.push(format!("{} {}", arg_type.to_token_stream(), arg_name));
            wrapper_args.push(format!("{} {}", arg_type.to_token_stream(), arg_name));
            arg_names.push(arg_name.to_string());
        }
        FunctionArgWrapper {
            arg_name,
            wrapper_type: FunctionArgWrapperType::String,
            ..
        } => {
            cpp_args.push(format!("std::string&& {arg_name}"));
            wrapper_args.push(format!("const char* {arg_name}"));
            arg_names.push(format!("casted_{arg_name}"));
            arg_casts.push(format!(r#"auto casted_{arg_name} = {arg_name}.data();"#));
        }
        FunctionArgWrapper {
            arg_name,
            arg_type,
            wrapper_type: FunctionArgWrapperType::Struct,
        } => {
            let struct_type = arg_type.to_token_stream().to_string();
            cpp_args.push(format!("const {struct_type}& {arg_name}"));
            wrapper_args.push(format!("void* {arg_name}"));
            arg_names.push(format!("casted_{arg_name}"));
            arg_casts.push(format!(
                r#"auto casted_{arg_name} = {arg_name}.self_ptr();"#
            ));
            includes.insert(format!("#include \"{struct_type}.h\""));
        }
    });

    let cpp_args = cpp_args.join(", ");
    let wrapper_args = wrapper_args.join(", ");
    let arg_names = arg_names.join(", ");
    let arg_casts = arg_casts.join("\n");
    let includes = includes.into_iter().collect::<HashSet<_>>();

    MappedCppFunctionArgsTokens {
        cpp_args,
        wrapper_args,
        arg_names,
        arg_casts,
        includes,
    }
}

pub fn gen_function_declaration(function_wrapper: &FunctionWrapper) -> String {
    let fn_name = function_wrapper.name.to_string();
    let extern_fn_name = &function_wrapper.extern_function_name;
    let MappedCppFunctionArgsTokens {
        cpp_args,
        wrapper_args,
        mut includes,
        ..
    } = map_args(function_wrapper.args_wrappers.iter());

    let ReturnTypes {
        ext_return_type,
        return_type,
        return_type_includes,
        ..
    } = map_return_type(&function_wrapper.return_wrapper);

    includes.insert(return_type_includes);
    let includes = includes.into_iter().collect::<Vec<_>>().join("\n");

    format!(
        r#"
#include "base.h"
{includes}

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
    let MappedCppFunctionArgsTokens {
        cpp_args,
        arg_names,
        arg_casts,
        ..
    } = map_args(function_wrapper.args_wrappers.iter());

    let ReturnTypes {
        return_type,
        return_cast,
        ext_return_type,
        ..
    } = map_return_type(&function_wrapper.return_wrapper);

    let arg_casts = prepend_each_line_with_n_tabs(&arg_casts, 1);
    let return_casts = prepend_each_line_with_n_tabs(&return_cast, 1);

    format!(
        r#"
#include "{fn_name}.h"

{return_type} {fn_name}({cpp_args}) {{
{arg_casts}
    {ext_return_type} result = {extern_fn_name}({arg_names});
{return_casts}
}}
"#
    )
}
