use std::collections::HashMap;

use crate::python::PYTHON_LIB_GETTER_NAME;
use crate::wrapper::python::{ClassCode, arg_cast, set_extern_fn_resttype, type_hint};
use crate::wrapper::{FieldWrapper, FieldWrapperType, StructWrapper};
use quote::ToTokens;
use syn::Type;

pub fn gen_class(struct_wrapper: &StructWrapper) -> ClassCode {
    let class_name = &struct_wrapper.name;

    let header = format!(
        r#"
class {class_name}:"#
    );

    let body = gen_body(struct_wrapper);

    let name = class_name.to_string();

    let imports = gen_imports(struct_wrapper);

    ClassCode {
        header,
        body,
        name,
        imports,
    }
}

fn gen_imports(struct_wrapper: &StructWrapper) -> HashMap<String, String> {
    let imports = HashMap::from([(
        PYTHON_LIB_GETTER_NAME.to_string(),
        format!("from .global_state import {PYTHON_LIB_GETTER_NAME}"),
    )]);

    struct_wrapper.fields.iter().fold(
        imports,
        |mut acc: HashMap<String, String>, field_wrapper| {
            match field_wrapper.wrapper_type {
                FieldWrapperType::Custom => {
                    let type_name = field_wrapper.field_type.to_token_stream().to_string();
                    acc.insert(
                        type_name.clone(),
                        format!("from .{type_name} import {type_name}"),
                    );
                }
                FieldWrapperType::String => {
                    acc.insert(
                        "FfiSlice".to_string(),
                        "from .global_state import FfiSlice".to_string(),
                    );
                }
                FieldWrapperType::Primitive => {
                    acc.insert("ctypes".to_string(), "import ctypes".to_string());
                }
            }
            acc
        },
    )
}

fn gen_body(struct_wrapper: &StructWrapper) -> String {
    let default_constructor = gen_default_constructor(struct_wrapper);
    let destructor = gen_destructor(struct_wrapper);
    let properties = gen_properties(struct_wrapper);
    let raw_ptr_method = gen_raw_ptr_method();

    format!(
        r#"{properties}
{default_constructor}
{raw_ptr_method}
{destructor}"#
    )
}

fn gen_destructor(struct_wrapper: &StructWrapper) -> String {
    let drop_ext_fn_name = &struct_wrapper.drop_ext_fn_name;
    format!(
        r#"
    def __del__(self):
        if self._self_ptr is not None:
            {PYTHON_LIB_GETTER_NAME}().{drop_ext_fn_name}(self._self_ptr)
            self._self_ptr = None"#
    )
}

fn gen_raw_ptr_method() -> String {
    "
    def raw_ptr(self):
        return self._self_ptr"
        .to_string()
}

fn gen_default_constructor(struct_wrapper: &StructWrapper) -> String {
    if let Some(default_constructor) = &struct_wrapper.default_constructor {
        let extern_fn_name = &default_constructor.extern_fn_name;
        format!(
            r#"
    def __init__(self, ptr = None):
        if not ptr:
            self._self_ptr = {PYTHON_LIB_GETTER_NAME}().{extern_fn_name}()
        else:
            self._self_ptr = ptr
"#
        )
    } else {
        "
    def __init__(self, ptr):
        self._self_ptr = ptr
"
        .to_string()
    }
}

fn gen_properties(struct_wrapper: &StructWrapper) -> String {
    struct_wrapper
        .fields
        .iter()
        .map(gen_property)
        .collect::<String>()
}

fn gen_property(field_wrapper: &FieldWrapper) -> String {
    let field_type = &field_wrapper.field_type;
    let field_name = &field_wrapper.field_name;

    let getter = if let Some(getter) = &field_wrapper.getter {
        let extern_fn_name = &getter.extern_fn_name;
        let type_hint = type_hint(field_type);
        let result_cast = prop_result_cast(&field_wrapper.field_type, "result");
        let set_extern_fn_resttype = set_extern_fn_resttype(field_type, extern_fn_name);

        format!(
            r#"    
    @property
    def {field_name}(self) -> {type_hint}:
        {set_extern_fn_resttype}
        result = {PYTHON_LIB_GETTER_NAME}().{extern_fn_name}(self._self_ptr)
        return {result_cast}"#
        )
    } else {
        String::new()
    };

    let setter = if let Some(setter) = &field_wrapper.setter {
        let extern_fn_name = &setter.extern_fn_name;
        let arg_cast = arg_cast(field_type, "value");

        format!(
            r#"
    @{field_name}.setter
    def {field_name}(self, value):
        {PYTHON_LIB_GETTER_NAME}().{extern_fn_name}(self._self_ptr, {arg_cast})"#
        )
    } else {
        String::new()
    };

    format!(
        r#"
{getter}
{setter}"#
    )
}

fn prop_result_cast(ty: &Type, result_var_name: &str) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            match segment.ident.to_string().as_str() {
                "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => result_var_name.to_string(),
                "bool" => format!("ctypes.c_byte({result_var_name}).value != 0"),
                "String" => format!("FfiSlice({result_var_name}).py_str()"),
                _ => format!("{}({})", ty.to_token_stream(), result_var_name),
            }
        }
        _ => unimplemented!("Result cast not implemented for this type"),
    }
}
