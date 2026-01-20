use core::panic;
use std::collections::HashMap;

use crate::prepend_each_line_with_n_tabs;
use crate::python::PYTHON_LIB_GETTER_NAME;
use crate::wrapper::python::{
    FunctionCode, result_cast_and_return, set_extern_fn_resttype, type_hint_from_wrapper_type,
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
            let type_hint_str = format!(
                " -> {}",
                type_hint_from_wrapper_type(&return_wrapper.wrapper_type)
            );
            let ret_expt = prepend_each_line_with_n_tabs(
                &result_cast_and_return(&return_wrapper.wrapper_type),
                1,
            );
            let set_restype = set_extern_fn_resttype(
                &return_wrapper.wrapper_type,
                &function.extern_function_name,
            );
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

                WrapperType::Vec(inner) => {
                    let arg_name = &arg_wrapper.arg_name;
                    let inner_type_name = inner.name();

                    // Create a vector wrapper and populate it
                    let cast_lines = vec![format!(
                        "casted_{arg_name} = {inner_type_name}Vec.from_list({arg_name})"
                    )];
                    casts.extend(cast_lines);
                    call_list.push(format!("casted_{arg_name}.raw_ptr()"));
                }

                WrapperType::Enum(_) => {
                    let arg_name = &arg_wrapper.arg_name;
                    let cast_line = format!("{arg_name}_ffi = {arg_name}.to_ffi()");
                    casts.push(cast_line);
                    call_list.push(format!("{arg_name}_ffi"));
                }

                WrapperType::Result(_) => {
                    panic!("Result types are not supported as function arguments in python");
                }

                WrapperType::UnitExpr => unreachable!(),
            }
            (casts, call_list)
        },
    )
}

fn gen_imports(function: &FunctionWrapper) -> HashMap<String, String> {
    let mut imports = function.args_wrappers.iter().fold(
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
                WrapperType::Enum(_) => {
                    let type_name = arg_wrapper.arg_type.to_token_stream().to_string();
                    acc.insert(
                        type_name.clone(),
                        format!("from .{type_name} import {type_name}"),
                    );
                }
                WrapperType::String => {
                    acc.insert("ctypes".to_string(), "import ctypes".to_string());
                }
                WrapperType::Vec(inner_type) => {
                    let inner_type_name = inner_type.name();
                    acc.insert("List".to_string(), "from typing import List".to_string());
                    acc.insert(
                        format!("{inner_type_name}Vec"),
                        format!("from .vec_{inner_type_name} import {inner_type_name}Vec"),
                    );
                }
                _ => {}
            }
            acc
        },
    );

    if let Some(return_wrapper) = &function.return_wrapper {
        match &return_wrapper.wrapper_type {
            WrapperType::Vec(inner) => {
                imports.insert(
                    format!("{}Vec", inner.name()),
                    format!("from .vec_{} import {}Vec", inner.name(), inner.name()),
                );
            }
            WrapperType::Result(inner) => {
                let inner_type_name = inner.name();
                imports.insert(
                    format!("{inner_type_name}Result"),
                    format!("from .result_{inner_type_name} import {inner_type_name}Result"),
                );
                imports.insert(
                    "RustException".to_string(),
                    "from .global_state import RustException".to_string(),
                );
            }
            _ => {}
        }
    }

    imports
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
                type_hint_from_wrapper_type(inner)
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
        WrapperType::Enum(_) => {
            format!(
                "{}: {}",
                arg_wrapper.arg_name,
                arg_wrapper.arg_type.to_token_stream()
            )
        }
        WrapperType::Result(_) => {
            panic!("Result types are not supported as function arguments in python");
        }
        WrapperType::UnitExpr => unreachable!(),
    }
}
