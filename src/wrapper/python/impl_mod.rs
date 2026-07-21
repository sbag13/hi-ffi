use std::collections::HashMap;
use std::ops::Deref;

use quote::ToTokens;

use crate::prepend_each_line_with_n_tabs;
use crate::python::PYTHON_LIB_GETTER_NAME;
use crate::wrapper::impl_block_wrapper::ImplBlockWrapper;
use crate::wrapper::python::{
    ClassCode, arg_cast, result_cast_and_return, set_extern_fn_resttype,
    type_hint_from_wrapper_type,
};
use crate::wrapper::{FunctionArgWrapper, WrapperType};

pub fn gen_methods_mod(impl_block: &ImplBlockWrapper) -> ClassCode {
    let class_name = impl_block.struct_name.to_string();

    let header = format!(
        r#"
class {class_name}:"#
    );

    let mut imports: HashMap<String, String> = HashMap::new();
    imports.insert(
        PYTHON_LIB_GETTER_NAME.to_string(),
        format!("from .global_state import {PYTHON_LIB_GETTER_NAME}"),
    );

    let mut body_sections: Vec<String> = vec![];

    // Ensure raw_ptr exists when only methods added (no struct translator ran)
    body_sections.push("\n    def raw_ptr(self):\n        return self._self_ptr".to_string());

    for method in impl_block.methods.iter().filter(|m| m.public) {
        let py_name = method.name.to_string();
        let extern_fn_name = &method.extern_function_name;

        let (call_args, pre_casts, py_args_sig) =
            call_args_and_pre_casts(&method.args, &mut imports);

        let call_target = if method.is_static {
            // static methods don't use self
            String::new()
        } else {
            "self._self_ptr, ".to_string()
        };

        // Return handling
        let mut ret_hint = String::new();
        let mut ret_line = String::new();
        let mut restype_set = String::new();
        let mut local_imports = String::new();
        if let Some(ret) = &method.return_wrapper {
            // Check if this is a vector return type and add proper imports

            match &ret.wrapper_type {
                WrapperType::Vec(inner) => {
                    insert_imports_for_vec_inner(inner.deref(), &mut imports);
                }
                WrapperType::Result(inner) => {
                    let inner_name = inner.name();
                    let result_type = format!("{inner_name}Result");
                    match inner.deref() {
                        // Avoid cyclic imports
                        WrapperType::Enum(n) | WrapperType::Struct(n) if n == &class_name => {
                            local_imports =
                                format!("from .result_{inner_name} import {result_type}");
                        }
                        _ => {
                            imports.insert(
                                result_type.clone(),
                                format!("from .result_{inner_name} import {result_type}"),
                            );
                        }
                    }

                    imports.insert(
                        "RustException".to_string(),
                        "from .global_state import RustException".to_string(),
                    );

                    match inner.deref() {
                        WrapperType::Enum(inner) | WrapperType::Struct(inner)
                            if inner != &class_name =>
                        {
                            imports
                                .insert(inner.to_string(), format!("from .{inner} import {inner}"));
                        }
                        WrapperType::Vec(inner) => {
                            insert_imports_for_vec_inner(inner.deref(), &mut imports);
                        }
                        _ => (),
                    }
                }
                WrapperType::Option(inner) => {
                    let inner_name = inner.name();
                    let option_type = format!("{inner_name}Option");
                    match inner.deref() {
                        // Avoid cyclic imports
                        WrapperType::Enum(n) | WrapperType::Struct(n) if n == &class_name => {
                            local_imports =
                                format!("from .option_{inner_name} import {option_type}");
                        }
                        _ => {
                            imports.insert(
                                option_type.clone(),
                                format!("from .option_{inner_name} import {option_type}"),
                            );
                        }
                    }

                    imports.insert(
                        "Optional".to_string(),
                        "from typing import Optional".to_string(),
                    );

                    match inner.deref() {
                        WrapperType::Enum(inner) | WrapperType::Struct(inner)
                            if inner != &class_name =>
                        {
                            imports
                                .insert(inner.to_string(), format!("from .{inner} import {inner}"));
                        }
                        WrapperType::Vec(inner) => {
                            insert_imports_for_vec_inner(inner.deref(), &mut imports);
                        }
                        _ => (),
                    }
                }
                WrapperType::Trait(trait_name) => {
                    imports.insert(
                        format!("{trait_name}Impl"),
                        format!("from . import {trait_name}"),
                    );
                }
                _ => (),
            }

            let self_ret = match &ret.wrapper_type {
                WrapperType::Result(inner)
                | WrapperType::Vec(inner)
                | WrapperType::Option(inner) => inner.name() == class_name,
                WrapperType::Struct(s_name) => s_name == &class_name,
                _ => false,
            };

            if self_ret {
                imports.insert("Self".to_string(), "from typing import Self".to_string());
                ret_hint = " -> Self".to_string();
            } else {
                ret_hint = format!(" -> {}", type_hint_from_wrapper_type(&ret.wrapper_type));
            };

            restype_set = set_extern_fn_resttype(&ret.wrapper_type, extern_fn_name);

            ret_line = if class_name == ret.wrapper_type.name() {
                prepend_each_line_with_n_tabs(&format!("return {}(result)", class_name), 2)
            } else {
                prepend_each_line_with_n_tabs(&result_cast_and_return(&ret.wrapper_type), 2)
            };
        }

        let recv_and_args = if method.is_static {
            py_args_sig.join(", ")
        } else if py_args_sig.is_empty() {
            "self".to_string()
        } else {
            format!("self, {}", py_args_sig.join(", "))
        };

        let decorator = if method.is_static {
            "@staticmethod\n    "
        } else {
            ""
        };

        let pre_casts = pre_casts
            .iter()
            .map(|s| prepend_each_line_with_n_tabs(s, 2))
            .collect::<Vec<String>>()
            .join("\n");

        let call_args = call_args.join(", ");

        let body = format!(
            r#"
    {decorator}def {py_name}({recv_and_args}){ret_hint}:
        {restype_set}
        {local_imports}
{pre_casts}
        result = {PYTHON_LIB_GETTER_NAME}().{extern_fn_name}({call_target}{call_args})
{ret_line}"#
        );

        body_sections.push(body);
    }

    ClassCode {
        header,
        body: body_sections.join("\n"),
        name: class_name,
        imports,
    }
}

fn insert_imports_for_vec_inner(inner: &WrapperType, imports: &mut HashMap<String, String>) {
    if let WrapperType::Vec(inner) = inner {
        imports.insert("List".to_string(), "from typing import List".to_string());
        let vec_name = format!("{}Vec", inner.name());
        imports.insert(
            vec_name.clone(),
            format!("from .vec_{} import {}", inner.name(), vec_name),
        );

        match inner.deref() {
            WrapperType::Enum(inner) | WrapperType::Struct(inner) => {
                imports.insert(inner.to_string(), format!("from .{inner} import {inner}"));
            }
            _ => {}
        }
    }
}

fn insert_imports_for_option_inner(inner: &WrapperType, imports: &mut HashMap<String, String>) {
    if let WrapperType::Option(inner) = inner {
        imports.insert(
            "Optional".to_string(),
            "from typing import Optional".to_string(),
        );
        let option_name = format!("{}Option", inner.name());
        imports.insert(
            option_name.clone(),
            format!("from .option_{} import {}", inner.name(), option_name),
        );
    }
}

pub(crate) fn call_args_and_pre_casts(
    args: &[FunctionArgWrapper],
    imports: &mut HashMap<String, String>,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut call_args: Vec<String> = vec![];
    let mut pre_casts: Vec<String> = vec![];
    let mut py_args_sig: Vec<String> = vec![];

    for arg in args {
        let arg_name = arg.arg_name.to_string();

        if arg.arg_type.to_token_stream().to_string().as_str() == "String" {
            imports.insert(
                "RustString".to_string(),
                "from .global_state import RustString".to_string(),
            );
        }

        py_args_sig.push(format!(
            "{}: {}",
            arg_name,
            type_hint_from_wrapper_type(&arg.wrapper_type)
        ));

        // For vector arguments, we need to store the wrapper object to prevent garbage collection
        match &arg.wrapper_type {
            WrapperType::Vec(_) => {
                insert_imports_for_vec_inner(&arg.wrapper_type, imports);

                pre_casts.push(format!(
                    "casted_{name}_wrapper = {cast}",
                    name = arg_name,
                    cast = arg_cast(&arg.arg_type, &arg_name)
                ));
                pre_casts.push(format!(
                    "casted_{name} = casted_{name}_wrapper.raw_ptr()",
                    name = arg_name
                ));
            }
            WrapperType::Option(_) => {
                insert_imports_for_option_inner(&arg.wrapper_type, imports);

                pre_casts.push(format!(
                    "casted_{name}_wrapper = {cast}",
                    name = arg_name,
                    cast = arg_cast(&arg.arg_type, &arg_name)
                ));
                pre_casts.push(format!(
                    "casted_{name} = casted_{name}_wrapper.raw_ptr()",
                    name = arg_name
                ));
            }
            WrapperType::Enum(inner_name) | WrapperType::Struct(inner_name) => {
                imports.insert(
                    inner_name.to_string(),
                    format!("from .{inner_name} import {inner_name}"),
                );
                pre_casts.push(format!(
                    "casted_{name} = {cast}",
                    name = arg_name,
                    cast = arg_cast(&arg.arg_type, &arg_name)
                ));
            }
            WrapperType::Trait(trait_name) => {
                imports.insert("Type".to_string(), "from typing import Type".to_string());
                imports.insert("ctypes".to_string(), "import ctypes".to_string());
                imports.insert(
                    trait_name.to_string(),
                    format!("from . import {trait_name}"),
                );
                pre_casts.push(format!(
                    r#"if not isinstance({arg_name}, {trait_name}.{trait_name}):
    raise TypeError(f"Object {{type({arg_name})}} does not implement {trait_name} protocol")
data_ptr = ctypes.c_void_p(id({arg_name}))
ctypes.pythonapi.Py_IncRef(data_ptr)
casted_{arg_name} = {trait_name}.{trait_name}Bridge(
    obj=data_ptr,
    vtable=ctypes.pointer({trait_name}.global{trait_name}VTable),
    deleter={trait_name}.global{trait_name}Deleter
)
"#,
                ));
            }
            _ => {
                pre_casts.push(format!(
                    "casted_{name} = {cast}",
                    name = arg_name,
                    cast = arg_cast(&arg.arg_type, &arg_name)
                ));
            }
        }

        call_args.push(format!("casted_{}", arg_name));
    }

    (call_args, pre_casts, py_args_sig)
}
