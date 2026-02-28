use std::collections::HashSet;

use crate::prepend_each_line_with_n_tabs;
use crate::wrapper::cpp::*;
use quote::ToTokens;

pub struct MappedCppFunctionArgsTokens {
    pub cpp_args: String,
    pub wrapper_args: String,
    pub call_args: String,
    pub arg_casts: String,
    pub from_rust_casts: String, // opposite of arg_casts, used for casting return values from Rust to C++
    pub includes: HashSet<String>,
}
pub fn map_args<'a>(
    args: impl Iterator<Item = &'a FunctionArgWrapper>,
) -> MappedCppFunctionArgsTokens {
    let (
        mut cpp_args,
        mut wrapper_args,
        mut call_args,
        mut arg_casts,
        mut from_rust_casts,
        mut includes,
    ): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, HashSet<String>) = (
        Vec::new(),
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
            wrapper_type:
                WrapperType::IntegerNumber(_) | WrapperType::Bool | WrapperType::FloatingPointNumber(_),
        } => {
            cpp_args.push(format!("{} {}", arg_type.to_token_stream(), arg_name));
            wrapper_args.push(format!("{} {}", arg_type.to_token_stream(), arg_name));
            call_args.push(arg_name.to_string());
        }

        FunctionArgWrapper {
            arg_name,
            wrapper_type: WrapperType::String,
            ..
        } => {
            cpp_args.push(format!("std::string {arg_name}"));
            wrapper_args.push(format!("char* {arg_name}"));
            call_args.push(format!("casted_{arg_name}"));
            arg_casts.push(format!(r#"auto casted_{arg_name} = {arg_name}.data();"#));
            from_rust_casts.push(format!(r#"std::string casted_{arg_name} = RustString({arg_name}).to_string();"#));
        }

        FunctionArgWrapper {
            arg_name,
            arg_type,
            wrapper_type: WrapperType::Struct(_),
        } => {
            let struct_type = arg_type.to_token_stream();
            cpp_args.push(format!("{struct_type}& {arg_name}"));
            wrapper_args.push(format!("void* {arg_name}"));
            call_args.push(format!("casted_{arg_name}"));
            arg_casts.push(format!(
                r#"auto casted_{arg_name} = {arg_name}.self_ptr();"#
            ));
            includes.insert(format!("#include \"{struct_type}.h\""));
            from_rust_casts.push(format!(
                r#"auto casted_{arg_name} = {struct_type}({arg_name});"#,
            ));
        }

        FunctionArgWrapper {
            wrapper_type: wt@ WrapperType::Vec( inner_wrapper) ,
            arg_name,
            ..
        } => {
            let arg_type = cpp_type(wt);
            cpp_args.push(format!("{arg_type}& {arg_name}"));
            includes.insert("#include <vector>".to_string());
            let inner_wrapper_name = inner_wrapper.name();
            includes.insert(format!(r#"#include "vec_{inner_wrapper_name}.h""#));
            arg_casts.push(format!("auto tmp_casted_{arg_name} = Rust{inner_wrapper_name}Vec::from_std({arg_name});
auto casted_{arg_name} = tmp_casted_{arg_name}.raw_ptr();"));
            call_args.push(format!("casted_{arg_name}"));
            wrapper_args.push(format!("void* {arg_name}"));
            from_rust_casts.push(format!("auto casted_{arg_name} = Rust{inner_wrapper_name}Vec({arg_name}).to_std();"));
        }

        FunctionArgWrapper {
            arg_name,
            arg_type,
            wrapper_type: WrapperType::Enum(_),
        } => {
            let enum_type = arg_type.to_token_stream();
            cpp_args.push(format!("{} {}", enum_type, arg_name));
            wrapper_args.push(format!("{} {}", enum_type, arg_name));
            call_args.push(arg_name.to_string());
            includes.insert(format!("#include \"{enum_type}.h\""));
        }

        FunctionArgWrapper {
            arg_name,
            wrapper_type: wt@ WrapperType::Option(inner),..
        } => {
            let arg_type =  cpp_type(wt);
            cpp_args.push(format!("{arg_type}& {arg_name}"));
            includes.insert("#include <optional>".to_string());
            let inner_wrapper_name = inner.name();
            includes.insert(format!(r#"#include "option_{inner_wrapper_name}.h""#));
            arg_casts.push(format!("auto tmp_casted_{arg_name} = Rust{inner_wrapper_name}Option::from_std({arg_name});
auto casted_{arg_name} = tmp_casted_{arg_name}.raw_ptr();"));
            call_args.push(format!("casted_{arg_name}"));
            wrapper_args.push(format!("void* {arg_name}"));
            from_rust_casts.push(format!("auto casted_{arg_name} = Rust{inner_wrapper_name}Option({arg_name}).to_std();"));
        }

        FunctionArgWrapper {
            wrapper_type: WrapperType::Result(_),
            ..
        } => {
            panic!("Result function arguments are not supported");
        }

        FunctionArgWrapper {
            wrapper_type: WrapperType::UnitExpr,
            ..
        } => {
            panic!("UnitExpr function arguments are not supported");
        }

        FunctionArgWrapper {
            wrapper_type: WrapperType::Trait(trait_name),
            arg_name,
            ..
        } => {
            cpp_args.push(format!("std::unique_ptr<{trait_name}>&& {arg_name}"));
            includes.insert(format!("#include \"{trait_name}.h\""));

            let uppercase_trait_name = trait_name.to_string().to_uppercase();
            let vtable_instance_name = format!("{}_VTABLE_INST", uppercase_trait_name);
            arg_casts.push(format!("{trait_name}Bridge bridge;
bridge.obj = {arg_name}.release();
bridge.vtable = &{vtable_instance_name};
bridge.deleter = [](void* obj) {{
    delete static_cast<{trait_name}*>(obj);
}};
"));
            call_args.push("bridge".to_string());
            wrapper_args.push(format!("{}Bridge {}_bridge", trait_name, arg_name));
            includes.insert("#include <memory>".to_string());
        }
    });

    let cpp_args = cpp_args.join(", ");
    let wrapper_args = wrapper_args.join(", ");
    let call_args = call_args.join(", ");
    let arg_casts = arg_casts.join("\n");
    let from_rust_casts = from_rust_casts.join("\n");

    MappedCppFunctionArgsTokens {
        cpp_args,
        wrapper_args,
        call_args,
        arg_casts,
        includes,
        from_rust_casts,
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

    includes.extend(return_type_includes);
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
        call_args: arg_names,
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
