use std::collections::HashMap;

use crate::python::PYTHON_LIB_GETTER_NAME;
use quote::ToTokens;
use syn::Type;

use crate::{Wrapper, wrapper::ParsedWrapper};

mod function;
mod impl_mod;
mod struct_mod;

pub struct PythonFiles {
    pub fn_code: Option<FunctionCode>,
    pub class_mod: Option<ClassCode>,
}

#[derive(Debug)]
pub struct ClassCode {
    pub header: String,
    pub body: String,
    pub name: String,
    pub imports: HashMap<String, String>,
}

pub struct FunctionCode {
    pub body: String,
    pub imports: HashMap<String, String>,
}

impl Wrapper {
    pub fn python(&self) -> PythonFiles {
        match &self.parsed {
            ParsedWrapper::Function(function_wrapper) => PythonFiles {
                fn_code: Some(function::gen_function(function_wrapper)),
                class_mod: None,
            },
            ParsedWrapper::Struct(struct_wrapper) => PythonFiles {
                fn_code: None,
                class_mod: Some(struct_mod::gen_class(struct_wrapper)),
            },
            ParsedWrapper::ImplBlock(impl_block_wrapper) => PythonFiles {
                fn_code: None,
                class_mod: Some(impl_mod::gen_methods_mod(impl_block_wrapper)),
            },
        }
    }
}

fn type_hint(field_type: &syn::Type) -> String {
    match field_type {
        syn::Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            match segment.ident.to_string().as_str() {
                "i32" | "i64" | "u32" | "u64" => "int".into(),
                "f32" | "f64" => "float".into(),
                "bool" => "bool".into(),
                "String" => "str".into(),
                _ => field_type.to_token_stream().to_string(),
            }
        }
        _ => unimplemented!("Type hint not implemented for this type"),
    }
}

fn result_cast(ty: &Type, result_var_name: &str) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            match segment.ident.to_string().as_str() {
                "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => format!("{result_var_name}"),
                "bool" => format!("ctypes.c_byte({result_var_name}).value != 0"),
                "String" => format!("RustString({result_var_name}).py_str()"),
                _ => format!("{}({})", ty.to_token_stream(), result_var_name),
            }
        }
        _ => unimplemented!("Result cast not implemented for this type"),
    }
}

fn arg_cast(ty: &Type, arg_name: &str) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            match segment.ident.to_string().as_str() {
                "i32" | "i64" | "u32" | "u64" => format!("{arg_name}"),
                "f32" => format!("ctypes.c_float({})", arg_name),
                "f64" => format!("ctypes.c_double({})", arg_name),
                "bool" => format!("ctypes.c_byte(1 if {} else 0)", arg_name),
                "String" => format!(r#"ctypes.c_char_p({arg_name}.encode("utf-8"))"#),
                _ => {
                    format!("{arg_name}.raw_ptr()")
                }
            }
        }
        _ => unimplemented!("Argument cast not implemented for this type"),
    }
}

fn set_extern_fn_resttype(ty: &Type, extern_fn_name: &str) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            match segment.ident.to_string().as_str() {
                "f32" => {
                    format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_float")
                }
                "f64" => {
                    format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_double")
                }
                "String" => {
                    format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_void_p")
                }
                _ => "".to_string(),
            }
        }
        _ => unimplemented!("Extern fn restype not implemented for this type"),
    }
}
