use std::collections::HashMap;

use crate::EXPORTED_SYMBOLS_PREFIX;
use crate::prepend_each_line_with_n_tabs;
use crate::wrapper::WrapperType;
use crate::wrapper::base::RUST_STRING_FROM_C_PTR_FN_NAME;
use crate::wrapper::python::impl_mod::call_args_and_pre_casts;
use crate::wrapper::python::{
    ClassCode, PYTHON_LIB_GETTER_NAME, c_type_from_wrapper_type, type_hint_from_wrapper_type,
};
use crate::wrapper::trait_wrapper::TraitWrapper;
use std::ops::Deref;

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


            let return_result = prepend_each_line_with_n_tabs(
                match &method.return_wrapper {
                    Some(return_type) => match &return_type.wrapper_type {
                        WrapperType::String => {
                            format!(r#"
result = py_obj.{method_name}({arg_names})
return {PYTHON_LIB_GETTER_NAME}().{RUST_STRING_FROM_C_PTR_FN_NAME}(ctypes.c_char_p(result.encode("utf-8")))
"#)
                        },
                        WrapperType::Struct(struct_name) => {
                            imports.insert(struct_name.clone(), format!("from .{struct_name} import {struct_name}"));
                            format!(r#"
result = py_obj.{method_name}({arg_names})
return result.leak()
"#)
                        }
                        WrapperType::Vec(inner) => {
                            let inner_name = inner.name();
                            imports.insert(format!("{inner_name}Vec"), format!("from .vec_{inner_name} import {inner_name}Vec"));
                            format!(r#"
result = py_obj.{method_name}({arg_names})
return {inner_name}Vec.from_list(result).leak()
"#)
                        }
                        WrapperType::Option(inner) => {
                            let inner_name = inner.name();
                            imports.insert(format!("{inner_name}Option"), format!("from .option_{inner_name} import {inner_name}Option"));
                            format!(r#"
result = py_obj.{method_name}({arg_names})
return {inner_name}Option.from_python(result).leak()"#
)
                        }
                        WrapperType::Enum(enum_name) => {
                            imports.insert(enum_name.clone(), format!("from .{enum_name} import {enum_name}"));
                            format!("return py_obj.{method_name}({arg_names})")
                        }
                        WrapperType::Result(inner) => {
                            let inner_name = inner.name();

                            let result_cast =  match inner.deref() {
                                WrapperType::String =>   r#"    rust_ok_arg = ctypes.c_char_p(python_result.encode("utf-8"))"#.to_string(),
                                WrapperType::Struct(_) => "    rust_ok_arg = python_result.raw_ptr()".to_string(),
                                WrapperType::Vec(inner) => format!("    rust_vec = {inner}Vec.from_list(python_result)
    rust_ok_arg = rust_vec.raw_ptr()", inner = inner.name()),
                                WrapperType::UnitExpr => "".to_string(),
                                _ => "    rust_ok_arg = python_result".to_string(),
                            };

                let rust_ok_arg = match inner.deref() {
                    WrapperType::UnitExpr => "".to_string(),
                    _ => "rust_ok_arg".to_string(),
                };

                let python_result_var_def = match inner.deref() {
                    WrapperType::UnitExpr => "".to_string(),
                    _ => "python_result = ".to_string(),
                };

                format!(
                    r#"try:
    {python_result_var_def}py_obj.{method_name}({arg_names})
{result_cast}
    return {PYTHON_LIB_GETTER_NAME}().{EXPORTED_SYMBOLS_PREFIX}{inner_name}_str_error_result_ok({rust_ok_arg})
except Exception as error:
    message = str(error)
    casted_msg = ctypes.c_char_p(message.encode("utf-8"))
    return {PYTHON_LIB_GETTER_NAME}().{EXPORTED_SYMBOLS_PREFIX}{inner_name}_str_error_result_err(casted_msg)
"#
                )
                        }
                        _ => format!("return py_obj.{method_name}({arg_names})"),
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
{return_result}

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


class {trait_name}Impl:
    def __init__(self, trait_ptr):
        self._trait_ptr = ctypes.c_void_p(trait_ptr)

    def __del__(self):
        if hasattr(self, '_trait_ptr'):
            {PYTHON_LIB_GETTER_NAME}().{EXPORTED_SYMBOLS_PREFIX}{trait_name}_BoxDyn_drop(self._trait_ptr)
"#
    );

    let impl_class_methods = trait_wrapper
        .functions
        .iter()
        .map(|method| {
            let method_name = &method.name;
            let boxdyn_func_name = format!("{}_BoxDyn", method.extern_function_name);

            let arg_names = method
                .args
                .iter()
                .map(|a| a.arg_name.to_string())
                .collect::<Vec<String>>()
                .join(", ");

            let args_signature = if method.args.is_empty() {
                "self".to_string()
            } else {
                format!("self, {arg_names}")
            };

            let ret_hint = if let Some(ret) = &method.return_wrapper {
                format!(" -> {}", type_hint_from_wrapper_type(&ret.wrapper_type))
            } else {
                String::new()
            };

            let arg_casts = prepend_each_line_with_n_tabs(
                &method
                    .args
                    .iter()
                    .filter_map(|arg| {
                        match &arg.wrapper_type {
                            WrapperType::String => {
                                Some(format!(
                                    "casted_{} = {} if isinstance({}, int) else {}.encode('utf-8')",
                                    arg.arg_name, arg.arg_name, arg.arg_name, arg.arg_name
                                ))
                            }
                            WrapperType::Enum(_) => {
                                Some(format!(
                                    "casted_{} = {}.to_ffi() if hasattr({}, 'to_ffi') else {}",
                                    arg.arg_name, arg.arg_name, arg.arg_name, arg.arg_name
                                ))
                            }
                            WrapperType::Struct(_) => {
                                Some(format!(
                                    "casted_{} = {}.raw_ptr() if hasattr({}, 'raw_ptr') else {}",
                                    arg.arg_name, arg.arg_name, arg.arg_name, arg.arg_name
                                ))
                            }
                            WrapperType::Vec(inner) => {
                                let inner_name = inner.name();
                                Some(format!(
                                    "casted_{arg_name} = {inner_name}Vec.from_list({arg_name})",
                                    arg_name = arg.arg_name
                                ))
                            }
                            WrapperType::Option(inner) => {
                                let inner_name = inner.name();
                                Some(format!(
                                    "casted_{arg_name} = {inner_name}Option.from_python({arg_name})",
                                    arg_name = arg.arg_name
                                ))
                            }
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                2,
            );

            let call_args = method
                .args
                .iter()
                .map(|arg| {
                    match &arg.wrapper_type {
                        WrapperType::String => format!("casted_{}", arg.arg_name),
                        WrapperType::Enum(_) => format!("ctypes.c_int(casted_{})", arg.arg_name),
                        WrapperType::Struct(_) => format!("casted_{}", arg.arg_name),
                        WrapperType::Vec(_) => format!("casted_{}.raw_ptr()", arg.arg_name),
                        WrapperType::Option(_) => format!("casted_{}.raw_ptr()", arg.arg_name),
                        _ => arg.arg_name.to_string(),
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            let call_line = if method.args.is_empty() {
                format!("result = {PYTHON_LIB_GETTER_NAME}().{}(self._trait_ptr)", boxdyn_func_name)
            } else {
                format!("result = {PYTHON_LIB_GETTER_NAME}().{}(self._trait_ptr, {})", boxdyn_func_name, call_args)
            };

            // Build argtypes for the function
            let mut arg_types = vec!["ctypes.c_void_p".to_string()];
            for arg in &method.args {
                arg_types.push(format!("ctypes.{}", c_type_from_wrapper_type(&arg.wrapper_type)));
            }
            let argtypes_str = format!("[{}]", arg_types.join(", "));

            let set_argtypes = format!(
                "{PYTHON_LIB_GETTER_NAME}().{boxdyn_func_name}.argtypes = {}",
                argtypes_str
            );

            // Build restype for the function
            let set_restype = if let Some(ret) = &method.return_wrapper {
                let ret_type = c_type_from_wrapper_type(&ret.wrapper_type);
                format!(
                    "{PYTHON_LIB_GETTER_NAME}().{boxdyn_func_name}.restype = ctypes.{ret_type}",
                )
            } else {
                format!("{PYTHON_LIB_GETTER_NAME}().{boxdyn_func_name}.restype = None")
            };

            let result_cast_str = if let Some(ret) = &method.return_wrapper {
                match &ret.wrapper_type {
                    WrapperType::String => {
                        imports.insert("RustString".to_string(), "from .global_state import RustString".to_string());
                        "return RustString(result).py_str()".to_string()
                    }
                    WrapperType::Enum(name) => {
                        imports.insert(name.clone(), format!("from . import {name}"));
                        format!("return {}.{}.from_ffi(result)", name, name)
                    }
                    WrapperType::Struct(name) => {
                        imports.insert(name.clone(), format!("from . import {name}"));
                        format!("return {}.{}(result)", name, name)
                    }
                    WrapperType::Vec(inner) => {
                        let inner_name = inner.name();
                        imports.insert(format!("{inner_name}Vec"), format!("from .vec_{inner_name} import {inner_name}Vec"));
                        format!("return {}Vec(result).to_list()", inner_name)
                    }
                    WrapperType::Option(inner) => {
                        let inner_name = inner.name();
                        imports.insert(format!("{inner_name}Option"), format!("from .option_{inner_name} import {inner_name}Option"));
                        format!("return {}Option(result).to_python()", inner_name)
                    }
                    WrapperType::Result(inner) => {
                        let inner_name = inner.name();

                        imports.insert(format!("{inner_name}Result"), format!("from .result_{inner_name} import {inner_name}Result"));
                        imports.insert("RustException".to_string(), "from .global_state import RustException".to_string());

                        format!(r#"
result = {inner_name}Result(result)
if result.is_err():
    raise RustException(result.unwrap_err())
else:
    return result.unwrap()
"#)
                    }
                    _ => "return result".to_string(),
                }
            } else {
                String::new()
            };

            let pre_casts = if arg_casts.is_empty() {
                String::new()
            } else {
                format!("{}\n", arg_casts)
            };

            let result_cast_str = prepend_each_line_with_n_tabs(&result_cast_str, 2);

            format!(
                r#"
    def {method_name}({args_signature}){ret_hint}:
        {set_argtypes}
        {set_restype}
{pre_casts}        {call_line}
{result_cast_str}"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let body = format!("{impl_class_methods}\n");

    ClassCode {
        header,
        body,
        name: trait_name.to_string(),
        imports,
    }
}
