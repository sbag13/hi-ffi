use std::collections::HashMap;
use std::ops::Deref;

use crate::python::PYTHON_LIB_GETTER_NAME;
use crate::wrapper::{WrapperType, is_enum_type};
use crate::{ReusableWrapper, prepend_each_line_with_n_tabs};
use quote::ToTokens;
use syn::Type;

use crate::wrapper::ParsedWrapper;
use crate::{EXPORTED_SYMBOLS_PREFIX, Wrapper};

mod enum_mod;
mod function;
mod impl_mod;
mod struct_mod;
mod trait_mod;

impl ReusableWrapper {
    pub fn python(&self) -> String {
        match self {
            ReusableWrapper::Vec(inner) => gen_vec_wrapper_python(inner),
            ReusableWrapper::Result(inner) => gen_result_wrapper_python(inner),
            ReusableWrapper::Option(inner) => gen_option_wrapper_python(inner),
        }
    }
}

pub fn gen_result_wrapper_python(inner: &WrapperType) -> String {
    let inner_name = inner.name();
    let inner_type_hint = type_hint_from_wrapper_type(inner);
    let inner_import = inner_import(inner);
    let is_err_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__is_err_{inner_name}_result");
    let unwrap_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__unwrap_{}_result", inner_name);
    let unwrap_ext_call = format!("{PYTHON_LIB_GETTER_NAME}().{unwrap_ext_name}(self._ptr)");

    let unwrap_val_cast = match inner {
        WrapperType::String => format!("result = RustString({unwrap_ext_call}).py_str();"),
        WrapperType::Struct(struct_name) => {
            format!("result = {struct_name}({unwrap_ext_call});")
        }
        WrapperType::Vec(vec_inner) => {
            let vec_inner_name = vec_inner.name();
            format!("result = {vec_inner_name}Vec({unwrap_ext_call}).to_list()")
        }
        WrapperType::Bool => format!("result = ctypes.c_byte({unwrap_ext_call}).value != 0"),
        WrapperType::Enum(inner) => {
            format!("result = {inner}.from_ffi({unwrap_ext_call})")
        }
        _ => format!("result = {unwrap_ext_call}"),
    };
    let unwrap_val_cast = prepend_each_line_with_n_tabs(&unwrap_val_cast, 2);

    format!(
        r#"
from .global_state import get_ffi_lib
import ctypes
{inner_import}

class {inner_name}Result:
    def __init__(self, ptr):
        self._ptr = ptr

    def is_err(self) -> bool:
        return ctypes.c_byte({PYTHON_LIB_GETTER_NAME}().{is_err_ext_fn_name}(self._ptr)).value != 0

    def unwrap(self) -> {inner_type_hint}:
{unwrap_val_cast}
        return result

    def unwrap_err(self):
        return {PYTHON_LIB_GETTER_NAME}().{EXPORTED_SYMBOLS_PREFIX}__unwrap_err_{inner_name}_result(self._ptr)
    
    def __del__(self):
        {PYTHON_LIB_GETTER_NAME}().{EXPORTED_SYMBOLS_PREFIX}__drop_{inner_name}_result(self._ptr)
"#
    )
}

pub fn gen_option_wrapper_python(inner: &WrapperType) -> String {
    let inner_name = inner.name();
    let inner_type_hint = type_hint_from_wrapper_type(inner);
    let inner_import = inner_import(inner);
    let is_some_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__is_some_{inner_name}_option");
    let unwrap_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__unwrap_{}_option", inner_name);
    let some_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__some_{}_option", inner_name);
    let none_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__none_{}_option", inner_name);
    let unwrap_ext_call = format!("{PYTHON_LIB_GETTER_NAME}().{unwrap_ext_name}(self._ptr)");

    let unwrap_val_cast = match inner {
        WrapperType::String => format!("result = RustString({unwrap_ext_call}).py_str();"),
        WrapperType::Struct(struct_name) => {
            format!("result = {struct_name}({unwrap_ext_call});")
        }
        WrapperType::Vec(vec_inner) => {
            let vec_inner_name = vec_inner.name();
            format!("result = {vec_inner_name}Vec({unwrap_ext_call}).to_list()")
        }
        WrapperType::Bool => format!("result = ctypes.c_byte({unwrap_ext_call}).value != 0"),
        WrapperType::Enum(inner) => {
            format!("result = {inner}.from_ffi({unwrap_ext_call})")
        }
        _ => format!("result = {unwrap_ext_call}"),
    };
    let unwrap_val_cast = prepend_each_line_with_n_tabs(&unwrap_val_cast, 2);

    let from_python_cast = match inner {
        WrapperType::String => "value.encode(\"utf-8\") if value is not None else None",
        WrapperType::Struct(_) => "value.raw_ptr() if value is not None else None",
        WrapperType::Bool => {
            "ctypes.c_byte(1 if value else 0).value if value is not None else None"
        }
        WrapperType::Enum(_) => "value.to_ffi() if value is not None else None",
        _ => "value if value is not None else None",
    };

    format!(
        r#"
from .global_state import get_ffi_lib
from typing import Optional
import ctypes
{inner_import}

class {inner_name}Option:
    def __init__(self, ptr):
        self._ptr = ptr

    def leak(self) -> ctypes.c_void_p:
        ret = self._ptr
        self._ptr = None
        return ret

    def raw_ptr(self):
        return self._ptr

    def is_some(self) -> bool:
        return ctypes.c_byte({PYTHON_LIB_GETTER_NAME}().{is_some_ext_fn_name}(self._ptr)).value != 0

    def unwrap(self) -> {inner_type_hint}:
{unwrap_val_cast}
        return result

    def to_python(self) -> Optional[{inner_type_hint}]:
        if self.is_some():
            return self.unwrap()
        else:
            return None

    @staticmethod
    def from_python(value: Optional[{inner_type_hint}]) -> '{inner_name}Option':
        if value is not None:
            casted_value = {from_python_cast}
            return {inner_name}Option({PYTHON_LIB_GETTER_NAME}().{some_ext_name}(casted_value))
        else:
            return {inner_name}Option({PYTHON_LIB_GETTER_NAME}().{none_ext_name}())
    
    def __del__(self):
        if self._ptr is not None:
            {PYTHON_LIB_GETTER_NAME}().{EXPORTED_SYMBOLS_PREFIX}__drop_{inner_name}_option(self._ptr)
"#
    )
}

pub fn gen_vec_wrapper_python(inner: &WrapperType) -> String {
    let type_name = inner.name();
    let inner_type_hint = type_hint_from_wrapper_type(inner);

    // Generate extern function names based on the pattern from wrapper.rs
    let drop_ext_fn_name = format!("{}__drop_{}_vec", crate::EXPORTED_SYMBOLS_PREFIX, type_name);
    let with_capacity_ext_fn_name = format!(
        "{}__with_capacity_{}_vec",
        crate::EXPORTED_SYMBOLS_PREFIX,
        type_name
    );
    let push_ext_fn_name = format!("{}__push_{}_vec", crate::EXPORTED_SYMBOLS_PREFIX, type_name);
    let len_ext_fn_name = format!("{}__len_{}_vec", crate::EXPORTED_SYMBOLS_PREFIX, type_name);
    let get_ext_fn_name = format!("{}__get_{}_vec", crate::EXPORTED_SYMBOLS_PREFIX, type_name);

    // Generate argument casting for push function
    let push_arg_cast = gen_vec_push_arg_cast(inner);

    // Generate result casting for get function
    let get_result_cast = gen_vec_get_result_cast(inner, &type_name);

    // Generate restype settings for extern functions
    let drop_restype = format!(
        "{}().{}.restype = None",
        crate::python::PYTHON_LIB_GETTER_NAME,
        drop_ext_fn_name
    );
    let with_capacity_restype = format!(
        "{}().{}.restype = ctypes.c_void_p",
        crate::python::PYTHON_LIB_GETTER_NAME,
        with_capacity_ext_fn_name
    );
    let len_restype = format!(
        "{}().{}.restype = ctypes.c_size_t",
        crate::python::PYTHON_LIB_GETTER_NAME,
        len_ext_fn_name
    );

    let get_restype = gen_vec_get_restype(inner, &get_ext_fn_name);

    let inner_import = inner_import(inner);

    format!(
        r#"
from typing import List
import ctypes
from .global_state import get_ffi_lib
{inner_import}

class {type_name}Vec:
    def __init__(self, ptr):
        self._ptr = ptr

    def raw_ptr(self):
        return self._ptr

    def leak(self) -> ctypes.c_void_p:
        ret = self._ptr
        self._ptr = None
        return ret

    @staticmethod
    def from_list(list: List[{inner_type_hint}]):
        length = len(list)
        {with_capacity_restype}
        ptr = {PYTHON_LIB_GETTER_NAME}().{with_capacity_ext_fn_name}(length)
        for i in range(length):
            value = list[i]
            {PYTHON_LIB_GETTER_NAME}().{push_ext_fn_name}(ptr, {push_arg_cast})
        return {type_name}Vec(ptr)
    
    def __del__(self):
        if hasattr(self, '_ptr') and self._ptr is not None:
            {drop_restype}
            {PYTHON_LIB_GETTER_NAME}().{drop_ext_fn_name}(self._ptr)
    
    def len(self) -> int:
        {len_restype}
        return {PYTHON_LIB_GETTER_NAME}().{len_ext_fn_name}(self._ptr)
    
    def __len__(self) -> int:
        return self.len()
    
    def get(self, index: int) -> {inner_type_hint}:
        {get_restype}
        result = {PYTHON_LIB_GETTER_NAME}().{get_ext_fn_name}(self._ptr, index)
        return {get_result_cast}
    
    def __getitem__(self, index: int) -> {inner_type_hint}:
        return self.get(index)
    
    def __iter__(self):
        for i in range(self.len()):
            yield self.get(i)
    
    def to_list(self) -> list:
        return [self.get(i) for i in range(self.len())]
"#
    )
}

/// Generate push argument casting for Vec elements
fn gen_vec_push_arg_cast(inner: &WrapperType) -> String {
    match inner {
        WrapperType::IntegerNumber(_) | WrapperType::Bool | WrapperType::FloatingPointNumber(_) => {
            "value".to_string()
        }
        WrapperType::String => "ctypes.c_char_p(value.encode(\"utf-8\"))".to_string(),
        WrapperType::Struct(_) => "value.raw_ptr()".to_string(),
        WrapperType::Vec(_) => panic!("Vec of vecs not supported yet!"),
        WrapperType::Enum(_) => "value.to_ffi()".to_string(),
        WrapperType::Result(_) => {
            panic!("Result types are not supported in Vec wrappers for python");
        }
        WrapperType::Option(_) => {
            panic!("Option types are not supported in Vec wrappers for python");
        }
        WrapperType::UnitExpr => unreachable!(),
        WrapperType::Trait(_) => panic!("Traits are not supported in Vec wrappers for python"),
    }
}

/// Generate get result casting for Vec elements
fn gen_vec_get_result_cast(inner: &WrapperType, type_name: &str) -> String {
    match inner {
        WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) => "result".to_string(),
        WrapperType::Bool => "ctypes.c_byte(result).value != 0".to_string(),
        WrapperType::String => "RustString(result).py_str()".to_string(),
        WrapperType::Struct(_) => {
            format!("{}(result)", type_name)
        }
        WrapperType::Vec(_) => panic!("Vec of vecs not supported yet!"),
        WrapperType::Enum(_) => {
            format!("{}.from_ffi(result)", type_name)
        }
        WrapperType::Result(_) => {
            panic!("Result types are not supported in Vec wrappers for python");
        }
        WrapperType::Option(_) => {
            panic!("Option types are not supported in Vec wrappers for python");
        }
        WrapperType::UnitExpr => unreachable!(),
        WrapperType::Trait(_) => panic!("Traits are not supported in Vec wrappers for python"),
    }
}

/// Generate restype for Vec::get method
fn gen_vec_get_restype(inner: &WrapperType, get_ext_fn_name: &str) -> String {
    match inner {
        WrapperType::IntegerNumber(r_int) => format!(
            "{PYTHON_LIB_GETTER_NAME}().{get_ext_fn_name}.restype = ctypes.{}",
            rust_int_to_c_types(r_int)
        ),
        WrapperType::FloatingPointNumber(r_float) => {
            if r_float == "f32" {
                format!("{PYTHON_LIB_GETTER_NAME}().{get_ext_fn_name}.restype = ctypes.c_float")
            } else {
                format!("{PYTHON_LIB_GETTER_NAME}().{get_ext_fn_name}.restype = ctypes.c_double")
            }
        }
        WrapperType::String | WrapperType::Struct(_) => {
            format!("{PYTHON_LIB_GETTER_NAME}().{get_ext_fn_name}.restype = ctypes.c_void_p")
        }
        WrapperType::Vec(_) => panic!("Vec of vecs not supported yet!"),
        WrapperType::Enum(_) | WrapperType::Bool => {
            format!("{PYTHON_LIB_GETTER_NAME}().{get_ext_fn_name}.restype = ctypes.c_int")
        }
        WrapperType::Result(_) => {
            panic!("Result types are not supported in Vec wrappers for python");
        }
        WrapperType::Option(_) => {
            panic!("Option types are not supported in Vec wrappers for python");
        }
        WrapperType::UnitExpr => unreachable!(),
        WrapperType::Trait(_) => panic!("Traits are not supported in Vec wrappers for python"),
    }
}

fn inner_import(inner: &WrapperType) -> String {
    match inner {
        WrapperType::Struct(name) => format!("from .{} import {}", name, name),
        WrapperType::String => "from .global_state import RustString".to_string(),
        WrapperType::Enum(name) => format!("from .{} import {}", name, name),
        WrapperType::Vec(inner) => {
            let inner_name = inner.name();

            let inner_import = match inner.deref() {
                WrapperType::Struct(_) => inner_import(inner),
                WrapperType::Enum(_) => inner_import(inner),
                _ => String::new(),
            };

            format!(
                "{inner_import}
from .vec_{inner_name} import {inner_name}Vec
from typing import List"
            )
        }
        _ => String::new(),
    }
}

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
            ParsedWrapper::Enum(enum_wrapper) => PythonFiles {
                fn_code: None,
                class_mod: Some(enum_mod::gen_enum_class(enum_wrapper)),
            },
            ParsedWrapper::Trait(trait_wrapper) => PythonFiles {
                fn_code: None,
                class_mod: Some(trait_mod::gen_trait_class(trait_wrapper)),
            },
        }
    }
}

fn type_hint_from_wrapper_type(wrapper_type: &crate::wrapper::WrapperType) -> String {
    match wrapper_type {
        WrapperType::IntegerNumber(_) => "int".into(),
        WrapperType::FloatingPointNumber(_) => "float".into(),
        WrapperType::Bool => "bool".into(),
        WrapperType::String => "str".into(),
        WrapperType::Struct(name) => name.to_string(),
        WrapperType::Vec(inner) => format!("List[{}]", type_hint_from_wrapper_type(inner)),
        WrapperType::Enum(name) => name.to_string(),
        WrapperType::Result(inner) => type_hint_from_wrapper_type(inner),
        WrapperType::Option(inner) => format!("Optional[{}]", type_hint_from_wrapper_type(inner)),
        WrapperType::UnitExpr => "None".to_string(),
        WrapperType::Trait(name) => format!("Type[{name}]"),
    }
}

fn type_hint_from_field_wrapper_type(wrapper_type: &WrapperType) -> String {
    match wrapper_type {
        WrapperType::IntegerNumber(_) => "int".into(),
        WrapperType::FloatingPointNumber(_) => "float".into(),
        WrapperType::Bool => "bool".into(),
        WrapperType::String => "str".into(),
        WrapperType::Struct(inner) => inner.into(),
        WrapperType::Vec(inner) => {
            format!("List[{}]", type_hint_from_wrapper_type(inner))
        }
        WrapperType::Option(inner) => {
            format!("Optional[{}]", type_hint_from_wrapper_type(inner))
        }
        WrapperType::Enum(inner) => inner.into(),
        WrapperType::Result(_) => {
            todo!("Result types are not supported as field types in Python wrappers")
        }
        WrapperType::UnitExpr => panic!("UnitExpr should not be used as a field type"),
        WrapperType::Trait(_) => panic!("Trait should not be used as a field type"),
    }
}

fn result_cast_and_return(wrapper: &WrapperType) -> String {
    match wrapper {
        WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) => {
            "return result".to_string()
        }
        WrapperType::Bool => "return ctypes.c_byte(result).value != 0".to_string(),
        WrapperType::String => "return RustString(result).py_str()".to_string(),
        WrapperType::Vec(inner_vec) => {
            // Use vector wrapper to convert Rust vector pointer to Python list
            let inner_type_str = inner_vec.name();
            format!(r#"return {inner_type_str}Vec(result).to_list()"#)
        }
        WrapperType::Enum(name) => {
            format!("return {}.from_ffi(result)", name)
        }
        WrapperType::Struct(ty) => {
            format!("return {}(result)", ty)
        }
        WrapperType::UnitExpr => "return None".to_string(),
        WrapperType::Result(inner) => {
            let inner_name = inner.name();
            format!(
                r#"rust_result = {inner_name}Result(result)
if rust_result.is_err():
    raise RustException(rust_result.unwrap_err())
else:
    return rust_result.unwrap()"#,
            )
        }
        WrapperType::Option(inner) => {
            let inner_name = inner.name();
            format!("return {inner_name}Option(result).to_python()")
        }
        WrapperType::Trait(trait_name) => {
            format!("return {trait_name}Impl(result)")
        }
    }
}

fn c_type_from_wrapper_type(wrapper: &WrapperType) -> &str {
    match wrapper {
        WrapperType::IntegerNumber(r_int) => rust_int_to_c_types(r_int),
        WrapperType::FloatingPointNumber(r_float) => {
            if r_float == "f32" {
                "c_float"
            } else {
                "c_double"
            }
        }
        WrapperType::Bool => "c_bool",
        WrapperType::String
        | WrapperType::Option(_)
        | WrapperType::Struct(_)
        | WrapperType::Result(_)
        | WrapperType::Trait(_)
        | WrapperType::Vec(_) => "c_void_p",
        WrapperType::Enum(_) => "c_int",
        WrapperType::UnitExpr => "c_void",
    }
}

fn arg_cast(ty: &Type, arg_name: &str) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            match segment.ident.to_string().as_str() {
                "i32" | "i64" | "u32" | "u64" => arg_name.to_string(),
                "f32" => format!("ctypes.c_float({})", arg_name),
                "f64" => format!("ctypes.c_double({})", arg_name),
                "bool" => format!("ctypes.c_byte(1 if {} else 0)", arg_name),
                "String" => format!(r#"ctypes.c_char_p({arg_name}.encode("utf-8"))"#),
                _ => {
                    // Check if it's a Vec type
                    if segment.ident.to_string().starts_with("Vec")
                        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                        && let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                        && let syn::Type::Path(inner_path) = inner_type
                        && let Some(inner_segment) = inner_path.path.segments.last()
                    {
                        let inner_type_name = inner_segment.ident.to_string();
                        format!("{}Vec.from_list({})", inner_type_name, arg_name)
                    } else if segment.ident.to_string().starts_with("Option")
                        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                        && let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                        && let syn::Type::Path(inner_path) = inner_type
                        && let Some(inner_segment) = inner_path.path.segments.last()
                    {
                        let inner_type_name = inner_segment.ident.to_string();
                        format!("{}Option.from_python({})", inner_type_name, arg_name)
                    } else {
                        // Check if it's an enum by looking for registered enum types
                        let type_name = ty.to_token_stream().to_string();
                        if is_enum_type(&type_name) {
                            format!("{}.to_ffi()", arg_name)
                        } else {
                            format!("{arg_name}.raw_ptr()")
                        }
                    }
                }
            }
        }
        _ => unimplemented!("Argument cast not implemented for this type"),
    }
}

fn set_extern_fn_resttype(wrapper: &WrapperType, extern_fn_name: &str) -> String {
    match wrapper {
        WrapperType::FloatingPointNumber(floating_type) => {
            if floating_type == "f32" {
                format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_float")
            } else {
                format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_double")
            }
        }
        WrapperType::String
        | WrapperType::Vec(_)
        | WrapperType::Result(_)
        | WrapperType::Option(_)
        | WrapperType::Struct(_) => {
            format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_void_p")
        }
        WrapperType::Enum(_name) => {
            format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_int")
        }
        WrapperType::Bool => {
            format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_int")
        }
        WrapperType::IntegerNumber(r_int) => {
            format!(
                "{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.{}",
                rust_int_to_c_types(r_int)
            )
        }
        WrapperType::UnitExpr => {
            format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_void")
        }
        WrapperType::Trait(_) => {
            format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_void_p")
        }
    }
}

fn rust_int_to_c_types(r_int: &str) -> &'static str {
    match r_int {
        "i8" => "c_int8",
        "i16" => "c_int16",
        "i32" => "c_int",
        "i64" => "c_int64",
        "u8" => "c_uint8",
        "u16" => "c_uint16",
        "u32" => "c_uint",
        "u64" => "c_uint64",
        _ => panic!("Int type not supported: {r_int}"),
    }
}

fn set_extern_fn_resttype_field(wrapper: &WrapperType, extern_fn_name: &str) -> String {
    match wrapper {
        WrapperType::IntegerNumber(_) => {
            format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_int")
        }
        WrapperType::FloatingPointNumber(r_float) => {
            if r_float == "f32" {
                format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_float")
            } else if r_float == "f64" {
                format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_double")
            } else {
                panic!("Unsupported floating point type for field")
            }
        }
        WrapperType::Bool | WrapperType::Enum(_) => {
            format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_int")
        }
        WrapperType::String
        | WrapperType::Struct(_)
        | WrapperType::Vec(_)
        | WrapperType::Result(_)
        | WrapperType::Trait(_)
        | WrapperType::Option(_) => {
            format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_void_p")
        }
        WrapperType::UnitExpr => {
            format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_void")
        }
    }
}
