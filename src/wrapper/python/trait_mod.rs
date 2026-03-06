use std::collections::HashMap;

use crate::prepend_each_line_with_n_tabs;
use crate::wrapper::WrapperType;
use crate::wrapper::base::RUST_STRING_FROM_C_PTR_FN_NAME;
use crate::wrapper::python::impl_mod::call_args_and_pre_casts;
use crate::wrapper::python::{
    ClassCode, PYTHON_LIB_GETTER_NAME, c_type_from_wrapper_type, type_hint_from_wrapper_type,
};
use crate::wrapper::trait_wrapper::TraitWrapper;

pub(crate) fn gen_trait_class(trait_wrapper: &TraitWrapper) -> ClassCode {
    let trait_name = &trait_wrapper.name;

    let mut imports = HashMap::from([
        (
            "Protocol".to_string(),
            "from typing import Protocol, runtime_checkable".to_string(),
        ),
        ("ctypes".to_string(), "import ctypes".to_string()),
        (
            PYTHON_LIB_GETTER_NAME.to_string(),
            format!("from .global_state import {PYTHON_LIB_GETTER_NAME}"),
        ),
    ]);

    let methods = prepend_each_line_with_n_tabs(
        &trait_wrapper
            .functions
            .iter()
            .map(|method| {
                let method_name = &method.name;

                let (_call_args, _pre_casts, py_args_sig) =
                    call_args_and_pre_casts(&method.args, &mut imports);

                let recv_and_args = if py_args_sig.is_empty() {
                    "self".to_string()
                } else {
                    format!("self, {}", py_args_sig.join(", "))
                };

                let ret_hint = if let Some(ret) = &method.return_wrapper {
                    format!(" -> {}", type_hint_from_wrapper_type(&ret.wrapper_type))
                } else {
                    String::new()
                };

                format!(
                    r#"
def {method_name}({recv_and_args}){ret_hint}:
    ...
"#
                )
            })
            .collect::<Vec<String>>()
            .join("\n"),
        1,
    );

    let vtable_funcs = trait_wrapper
        .functions
        .iter()
        .map(|method| {
            let method_name = &method.name;
            let args = method
                .args
                .iter()
                .map(|arg| {
                    let c_type = c_type_from_wrapper_type(&arg.wrapper_type);
                    format!(", ctypes.{c_type}")
                })
                .collect::<String>();

            let args = format!("ctypes.c_void_p{args}");

            let args = if let Some(ret) = &method.return_wrapper {
                let ret_c_type = c_type_from_wrapper_type(&ret.wrapper_type);
                format!("ctypes.{ret_c_type}, {args}")
            } else {
                format!("None, {args}")
            };

            let arg_names = method
                .args
                .iter()
                .map(|a| a.arg_name.to_string())
                .collect::<Vec<String>>()
                .join(", ");

            let args_signature =
                format!("obj, {arg_names}")
            ;

            let args_casts = prepend_each_line_with_n_tabs(
                &method
                    .args
                    .iter()
                    .map(|a| {
                        let arg_name = &a.arg_name;
                        match &a.wrapper_type {
                            WrapperType::String => {
                                format!("{arg_name} = RustString({arg_name}).py_str()")
                            }
                            WrapperType::Struct(struct_name) => {
                                format!("{arg_name} = {struct_name}({arg_name})")
                            }
                            WrapperType::Vec(inner) => {
                                let inner_name = inner.name();
                                format!("{arg_name} = {inner_name}Vec({arg_name}).to_list()")
                            }
                            WrapperType::Option(inner) => {
                                let inner_name = inner.name();
                                format!("{arg_name} = {inner_name}Option({arg_name}).to_python()")
                            }
                            _ => "".to_string(),
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
                1,
            );

            let result_cast = prepend_each_line_with_n_tabs(
                match &method.return_wrapper {
                    Some(return_type) => match &return_type.wrapper_type {
                        WrapperType::String => {
                            format!(r#"result = {PYTHON_LIB_GETTER_NAME}().{RUST_STRING_FROM_C_PTR_FN_NAME}(ctypes.c_char_p(result.encode("utf-8")))"#)
                        },
                        WrapperType::Struct(struct_name) => {
                            imports.insert(struct_name.clone(), format!("from .{struct_name} import {struct_name}"));
                            "result = result.leak()".to_string()
                        }
                        WrapperType::Vec(inner) => {
                            let inner_name = inner.name();
                            imports.insert(format!("{inner_name}Vec"), format!("from .vec_{inner_name} import {inner_name}Vec"));
                            format!("result = {inner_name}Vec.from_list(result).leak()")
                        }
                        WrapperType::Option(inner) => {
                            let inner_name = inner.name();
                            imports.insert(format!("{inner_name}Option"), format!("from .option_{inner_name} import {inner_name}Option"));
                            format!("result = {inner_name}Option.from_python(result).leak()")
                        }
                        WrapperType::Enum(enum_name) => {
                            imports.insert(enum_name.clone(), format!("from .{enum_name} import {enum_name}"));
                            "".to_string()
                        }
                        _ => "".to_string(),
                    },
                    None => "".to_string(),
                }
                .as_str(),
                1,
            );

            format!(
                r#"
def py_{method_name}({args_signature}):
    py_obj = ctypes.cast(obj, ctypes.py_object).value
{args_casts}
    result = py_obj.{method_name}({arg_names})
{result_cast}
    return result

_{method_name}_func_type = ctypes.CFUNCTYPE({args})
_{method_name}_func = ctypes.CFUNCTYPE({args})(py_{method_name})"#
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let vtable_fields = trait_wrapper
        .functions
        .iter()
        .map(|method| {
            let method_name = &method.name;
            format!(r#"        ("{method_name}", _{method_name}_func_type)"#)
        })
        .collect::<Vec<String>>()
        .join(",\n");

    let vtable_field_inits = trait_wrapper
        .functions
        .iter()
        .map(|method| {
            let method_name = &method.name;
            format!(r#"    {method_name}=_{method_name}_func"#)
        })
        .collect::<Vec<String>>()
        .join(",\n");

    let upper_case_trait_name = trait_name.to_string().to_uppercase();

    let header = format!(
        r#"
{vtable_funcs}

def py_{trait_name}_deleter(data_ptr):
    ctypes.pythonapi.Py_DecRef(ctypes.cast(data_ptr, ctypes.py_object))

{upper_case_trait_name}_DELETER_FUNC = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
global{trait_name}Deleter = {upper_case_trait_name}_DELETER_FUNC(py_{trait_name}_deleter)

class {trait_name}VTable(ctypes.Structure):
    _fields_ = [
{vtable_fields}
    ]

class {trait_name}Bridge(ctypes.Structure):
    _fields_ = [
        ("obj", ctypes.c_void_p),
        ("vtable", ctypes.POINTER({trait_name}VTable)),
        ("deleter", ctypes.CFUNCTYPE(None, ctypes.c_void_p)),
    ]


global{trait_name}VTable = {trait_name}VTable(
{vtable_field_inits}
)


@runtime_checkable
class {trait_name}(Protocol):
{methods}
"#
    );

    ClassCode {
        header,
        body: String::new(),
        name: trait_name.to_string(),
        imports,
    }
}
