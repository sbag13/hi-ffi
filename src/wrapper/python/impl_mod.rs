use std::collections::HashMap;

use quote::ToTokens;

use crate::python::PYTHON_LIB_GETTER_NAME;
use crate::wrapper::impl_block_wrapper::ImplBlockWrapper;
use crate::wrapper::python::{
    ClassCode, arg_cast, set_extern_fn_resttype, type_hint_from_wrapper_type,
};

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

        // Build python signature receivers
        let mut py_args_sig: Vec<String> = vec![];

        // map args
        let mut call_args: Vec<String> = vec![];
        let mut pre_casts: Vec<String> = vec![];
        for arg in &method.args {
            let arg_name = arg.arg_name.to_string();

            if arg.arg_type.to_token_stream().to_string().as_str() == "String" {
                imports.insert(
                    "RustString".to_string(),
                    "from .global_state import RustString".to_string(),
                );
            }

            // Check if this is a vector argument and add List import
            if let crate::wrapper::WrapperType::Vec(_) = &arg.wrapper_type {
                imports.insert("List".to_string(), "from typing import List".to_string());
            }

            py_args_sig.push(format!(
                "{}: {}",
                arg_name,
                type_hint_from_wrapper_type(&arg.wrapper_type)
            ));

            // For vector arguments, we need to store the wrapper object to prevent garbage collection
            if let crate::wrapper::WrapperType::Vec(_) = &arg.wrapper_type {
                pre_casts.push(format!(
                    "casted_{name}_wrapper = {cast}",
                    name = arg_name,
                    cast = arg_cast(&arg.arg_type, &arg_name)
                ));
                pre_casts.push(format!(
                    "casted_{name} = casted_{name}_wrapper.raw_ptr()",
                    name = arg_name
                ));
            } else {
                pre_casts.push(format!(
                    "casted_{name} = {cast}",
                    name = arg_name,
                    cast = arg_cast(&arg.arg_type, &arg_name)
                ));
            }

            call_args.push(format!("casted_{}", arg_name));
        }

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
        if let Some(ret) = &method.return_wrapper {
            // Check if this is a vector return type and add proper imports
            if let crate::wrapper::WrapperType::Vec(inner) = &ret.wrapper_type {
                imports.insert("List".to_string(), "from typing import List".to_string());
                let vec_name = format!("{}Vec", inner.name());
                imports.insert(
                    vec_name.clone(),
                    format!("from .vec_{} import {}", inner.name(), vec_name),
                );
            }

            ret_hint = format!(" -> {}", type_hint_from_wrapper_type(&ret.wrapper_type));
            restype_set = set_extern_fn_resttype(&ret.wrapper_type, extern_fn_name);
            ret_line = crate::wrapper::python::result_cast_and_return(&ret.wrapper_type);
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

        let body = format!(
            r#"
    {decorator}def {py_name}({recv_and_args}){ret_hint}:
        {restype_set}
        {pre_casts}
        result = {PYTHON_LIB_GETTER_NAME}().{extern_fn_name}({call_target}{call_args})
        {ret_line}"#,
            pre_casts = if pre_casts.is_empty() {
                "".into()
            } else {
                pre_casts.join("\n        ")
            },
            call_args = call_args.join(", "),
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
