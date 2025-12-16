use std::collections::HashMap;

use crate::python::PYTHON_LIB_GETTER_NAME;
use crate::wrapper::python::{
    FunctionCode, result_cast, set_extern_fn_resttype, type_hint, type_hint_from_str,
};
use crate::wrapper::{FunctionWrapper, WrapperType};
use quote::ToTokens;

pub fn gen_function(function: &FunctionWrapper) -> FunctionCode {
    let fn_name = &function.name;
    let extern_fn_name = &function.extern_function_name;
    let received_args = received_args(function);
    let (arg_casts, call_args_list) = args(function);
    let arg_casts = arg_casts.join("\n    ");
    let call_args_list = call_args_list.join(", ");
    let (ret_hint, return_expression, set_restype) = return_expression(function);

    let body = format!(
        r#"def {fn_name}({received_args}){ret_hint}:
    {set_restype}
    {arg_casts}
    result = {PYTHON_LIB_GETTER_NAME}().{extern_fn_name}({call_args_list})
    {return_expression}
"#
    );

    let imports = gen_imports(function);

    FunctionCode { body, imports }
}

fn return_expression(function: &FunctionWrapper) -> (String, String, String) {
    match &function.return_wrapper {
        Some(return_wrapper) => {
            let type_hint_str = format!(" -> {}", type_hint(&return_wrapper.return_type));
            let ret_expt = format!(
                "return {}",
                result_cast(&return_wrapper.return_type, "result")
            );
            let set_restype =
                set_extern_fn_resttype(&return_wrapper.return_type, &function.extern_function_name);
            (type_hint_str, ret_expt, set_restype)
        }
        None => ("".to_string(), "".to_string(), "".to_string()),
    }
}

fn args(function: &FunctionWrapper) -> (Vec<String>, Vec<String>) {
    function.args_wrappers.iter().fold(
        (vec![], vec![]),
        |(mut casts, mut call_list), arg_wrapper| {
            match &arg_wrapper.wrapper_type {
                WrapperType::IntegerNumber(_)
                | WrapperType::Bool
                | WrapperType::FloatingPointNumber(_) => {
                    call_list.push(arg_wrapper.arg_name.to_string())
                }

                WrapperType::String => {
                    let cast_line = format!(
                        "casted_{} = ctypes.c_char_p({}.encode(\"utf-8\"))",
                        arg_wrapper.arg_name, arg_wrapper.arg_name
                    );
                    casts.push(cast_line);
                    call_list.push(format!("casted_{}", arg_wrapper.arg_name));
                }

                WrapperType::Struct { .. } => {
                    let arg_name = &arg_wrapper.arg_name;
                    let cast_line = format!("{arg_name}_ptr = {arg_name}.raw_ptr()");
                    casts.push(cast_line);
                    call_list.push(format!("{arg_name}_ptr"));
                }

                WrapperType::Vec(_) => {
                    // TODO
                }
            }
            (casts, call_list)
        },
    )
}

fn gen_imports(function: &FunctionWrapper) -> HashMap<String, String> {
    function.args_wrappers.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<String, String>, arg_wrapper| {
            let type_name = arg_wrapper.arg_type.to_token_stream().to_string();
            match &arg_wrapper.wrapper_type {
                WrapperType::Struct(_) => {
                    acc.insert(
                        type_name.clone(),
                        format!("from .{type_name} import {type_name}"),
                    );
                }
                WrapperType::String => {
                    acc.insert("ctypes".to_string(), "import ctypes".to_string());
                }
                WrapperType::Vec(_) => {
                    acc.insert("List".to_string(), "from typing import List".to_string());
                }
                _ => {}
            }
            acc
        },
    )
}

fn received_args(function: &FunctionWrapper) -> String {
    function
        .args_wrappers
        .iter()
        .map(arg_receiver)
        .collect::<Vec<String>>()
        .join(", ")
}

fn arg_receiver(arg_wrapper: &crate::wrapper::FunctionArgWrapper) -> String {
    match &arg_wrapper.wrapper_type {
        WrapperType::Vec(inner) => {
            format!(
                "{}: List[{}]",
                arg_wrapper.arg_name,
                type_hint_from_str(&inner.name())
            )
        }
        WrapperType::IntegerNumber(_) => format!("{}: int", arg_wrapper.arg_name),
        WrapperType::FloatingPointNumber(_) => format!("{}: float", arg_wrapper.arg_name),
        WrapperType::Bool => format!("{}: bool", arg_wrapper.arg_name),
        WrapperType::String => format!("{}: str", arg_wrapper.arg_name),
        WrapperType::Struct { .. } => format!(
            "{}: {}",
            arg_wrapper.arg_name,
            arg_wrapper.arg_type.to_token_stream()
        ),
    }
}
