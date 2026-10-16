use std::collections::{HashMap, HashSet};

use crate::python::PYTHON_LIB_GETTER_NAME;
use crate::wrapper::WrapperType;
use crate::{ReusableWrapper, prepend_each_line_with_n_tabs};

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
            ReusableWrapper::HashMap(key, value) => gen_hashmap_wrapper_python(key, value),
        }
    }
}

pub fn gen_result_wrapper_python(inner: &WrapperType) -> String {
    let inner_name = inner.name();
    let inner_type_hint = type_hint_from_wrapper_type(inner);
    let inner_import = inner_imports(inner)
        .into_iter()
        .collect::<Vec<String>>()
        .join("\n");
    let is_err_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}is_err_{inner_name}_result");
    let unwrap_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}unwrap_{}_result", inner_name);
    let unwrap_ext_call = format!("{PYTHON_LIB_GETTER_NAME}().{unwrap_ext_name}(self._ptr)");
    let unwrap_restype = match inner {
        WrapperType::UnitExpr => {
            format!("{PYTHON_LIB_GETTER_NAME}().{unwrap_ext_name}.restype = None")
        }
        _ => format!(
            "{PYTHON_LIB_GETTER_NAME}().{unwrap_ext_name}.restype = ctypes.{}",
            c_type_from_wrapper_type(inner)
        ),
    };

    let unwrap_val_cast = match inner {
        WrapperType::String => format!("result = RustString({unwrap_ext_call}).py_str();"),
        WrapperType::Struct(struct_name) => {
            format!("result = {struct_name}.{struct_name}({unwrap_ext_call});")
        }
        WrapperType::Vec(_) => format!(
            "result = {}({unwrap_ext_call}).to_list()",
            python_container_class_name(inner)
        ),
        WrapperType::Option(_) => format!(
            "result = {}({unwrap_ext_call}).to_python()",
            python_container_class_name(inner)
        ),
        WrapperType::HashMap(_, _) => format!(
            "result = {}({unwrap_ext_call}).to_dict()",
            python_container_class_name(inner)
        ),
        WrapperType::Bool => format!("result = ctypes.c_byte({unwrap_ext_call}).value != 0"),
        WrapperType::Enum(inner) => {
            format!("result = {inner}.{inner}.from_ffi({unwrap_ext_call})")
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
        {unwrap_restype}
{unwrap_val_cast}
        return result

    def unwrap_err(self):
        return {PYTHON_LIB_GETTER_NAME}().{EXPORTED_SYMBOLS_PREFIX}unwrap_err_{inner_name}_result(self._ptr)
    
    def __del__(self):
        {PYTHON_LIB_GETTER_NAME}().{EXPORTED_SYMBOLS_PREFIX}drop_{inner_name}_result(self._ptr)
"#
    )
}

pub fn gen_option_wrapper_python(inner: &WrapperType) -> String {
    let inner_name = inner.name();
    let inner_type_hint = type_hint_from_wrapper_type(inner);
    let inner_import = inner_imports(inner)
        .into_iter()
        .collect::<Vec<String>>()
        .join("\n");
    let is_some_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}is_some_{inner_name}_option");
    let unwrap_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}unwrap_{}_option", inner_name);
    let some_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}some_{}_option", inner_name);
    let none_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}none_{}_option", inner_name);
    let unwrap_ext_call = format!("{PYTHON_LIB_GETTER_NAME}().{unwrap_ext_name}(self._ptr)");
    let unwrap_restype = format!(
        "{PYTHON_LIB_GETTER_NAME}().{unwrap_ext_name}.restype = ctypes.{}",
        c_type_from_wrapper_type(inner)
    );

    let unwrap_val_cast = match inner {
        WrapperType::String => format!("result = RustString({unwrap_ext_call}).py_str();"),
        WrapperType::Struct(struct_name) => {
            format!("result = {struct_name}.{struct_name}({unwrap_ext_call});")
        }
        WrapperType::Vec(_) => format!(
            "result = {}({unwrap_ext_call}).to_list()",
            python_container_class_name(inner)
        ),
        WrapperType::Option(_) => format!(
            "result = {}({unwrap_ext_call}).to_python()",
            python_container_class_name(inner)
        ),
        WrapperType::HashMap(_, _) => format!(
            "result = {}({unwrap_ext_call}).to_dict()",
            python_container_class_name(inner)
        ),
        WrapperType::Bool => format!("result = ctypes.c_byte({unwrap_ext_call}).value != 0"),
        WrapperType::Enum(inner) => {
            format!("result = {inner}.{inner}.from_ffi({unwrap_ext_call})")
        }
        _ => format!("result = {unwrap_ext_call}"),
    };
    let unwrap_val_cast = prepend_each_line_with_n_tabs(&unwrap_val_cast, 2);

    let from_python_cast = match inner {
        WrapperType::String => "casted_value = value.encode(\"utf-8\")".to_string(),
        WrapperType::Struct(_) => "casted_value = value.raw_ptr()".to_string(),
        WrapperType::Bool => "casted_value = ctypes.c_byte(1 if value else 0).value".to_string(),
        WrapperType::Enum(_) => "casted_value = value.to_ffi()".to_string(),
        WrapperType::Vec(_) => format!(
            "nested_wrapper = {}.from_list(value)\n            casted_value = nested_wrapper.raw_ptr()",
            python_container_class_name(inner)
        ),
        WrapperType::Option(_) => format!(
            "nested_wrapper = {}.from_python(value)\n            casted_value = nested_wrapper.raw_ptr()",
            python_container_class_name(inner)
        ),
        WrapperType::HashMap(_, _) => format!(
            "nested_wrapper = {}.from_dict(value)\n            casted_value = nested_wrapper.raw_ptr()",
            python_container_class_name(inner)
        ),
        _ => "casted_value = value".to_string(),
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
        {unwrap_restype}
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
            {from_python_cast}
            return {inner_name}Option({PYTHON_LIB_GETTER_NAME}().{some_ext_name}(casted_value))
        else:
            return {inner_name}Option({PYTHON_LIB_GETTER_NAME}().{none_ext_name}())
    
    def __del__(self):
        if self._ptr is not None:
            {PYTHON_LIB_GETTER_NAME}().{EXPORTED_SYMBOLS_PREFIX}drop_{inner_name}_option(self._ptr)
"#
    )
}

pub fn gen_vec_wrapper_python(inner: &WrapperType) -> String {
    let type_name = inner.name();
    let inner_type_hint = type_hint_from_wrapper_type(inner);

    // Generate extern function names based on the pattern from wrapper.rs
    let drop_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}drop_{type_name}_vec");
    let with_capacity_ext_fn_name =
        format!("{EXPORTED_SYMBOLS_PREFIX}with_capacity_{type_name}_vec",);
    let push_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}push_{type_name}_vec");
    let len_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}len_{type_name}_vec");
    let get_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}get_{type_name}_vec");

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

    let inner_import = inner_imports(inner)
        .into_iter()
        .collect::<Vec<String>>()
        .join("\n");

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
        WrapperType::IntegerNumber(_) | WrapperType::Bool => "value".to_string(),
        WrapperType::FloatingPointNumber(name) if name == "f32" => {
            "ctypes.c_float(value)".to_string()
        }
        WrapperType::FloatingPointNumber(_) => "ctypes.c_double(value)".to_string(),
        WrapperType::String => "ctypes.c_char_p(value.encode(\"utf-8\"))".to_string(),
        WrapperType::Struct(_) => "value.raw_ptr()".to_string(),
        WrapperType::Vec(_) => format!(
            "(nested_wrapper := {}.from_list(value)).raw_ptr()",
            python_container_class_name(inner)
        ),
        WrapperType::Option(_) => format!(
            "(nested_wrapper := {}.from_python(value)).raw_ptr()",
            python_container_class_name(inner)
        ),
        WrapperType::HashMap(_, _) => format!(
            "(nested_wrapper := {}.from_dict(value)).raw_ptr()",
            python_container_class_name(inner)
        ),
        WrapperType::Enum(_) => "value.to_ffi()".to_string(),
        WrapperType::Result(_) => {
            panic!("Result types are not supported in Vec wrappers for python");
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
            format!("{}.{}(result)", type_name, type_name)
        }
        WrapperType::Vec(_) => format!("{}(result).to_list()", python_container_class_name(inner)),
        WrapperType::Option(_) => {
            format!("{}(result).to_python()", python_container_class_name(inner))
        }
        WrapperType::HashMap(_, _) => {
            format!("{}(result).to_dict()", python_container_class_name(inner))
        }
        WrapperType::Enum(_) => {
            format!("{}.{}.from_ffi(result)", type_name, type_name)
        }
        WrapperType::Result(_) => {
            panic!("Result types are not supported in Vec wrappers for python");
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
        WrapperType::Vec(_) | WrapperType::Option(_) | WrapperType::HashMap(_, _) => {
            format!("{PYTHON_LIB_GETTER_NAME}().{get_ext_fn_name}.restype = ctypes.c_void_p")
        }
        WrapperType::Enum(_) | WrapperType::Bool => {
            format!("{PYTHON_LIB_GETTER_NAME}().{get_ext_fn_name}.restype = ctypes.c_int")
        }
        WrapperType::Result(_) => {
            panic!("Result types are not supported in Vec wrappers for python");
        }
        WrapperType::UnitExpr => unreachable!(),
        WrapperType::Trait(_) => panic!("Traits are not supported in Vec wrappers for python"),
    }
}

fn inner_imports(inner: &WrapperType) -> HashSet<String> {
    match inner {
        WrapperType::Struct(name) => HashSet::from([format!("from . import {name}")]),
        WrapperType::Enum(name) => HashSet::from([format!("from . import {name}")]),
        WrapperType::String => HashSet::from(["from .global_state import RustString".to_string()]),
        WrapperType::Vec(inner) => {
            let mut imports = inner_imports(inner);
            imports.insert(format!(
                "from .vec_{} import {}",
                inner.name(),
                python_container_class_name(&WrapperType::Vec(inner.clone()))
            ));
            imports.insert("from typing import List".to_string());
            imports
        }
        WrapperType::Option(inner) => {
            let mut imports = inner_imports(inner);
            imports.insert(format!(
                "from .option_{} import {}",
                inner.name(),
                python_container_class_name(&WrapperType::Option(inner.clone()))
            ));
            imports.insert("from typing import Optional".to_string());
            imports
        }
        WrapperType::HashMap(key, value) => {
            let mut imports = inner_imports(key);
            imports.extend(inner_imports(value));
            imports.insert(format!(
                "from .hashmap_{}_{} import {}",
                key.name(),
                value.name(),
                python_container_class_name(&WrapperType::HashMap(key.clone(), value.clone()))
            ));
            imports.insert("from typing import Dict".to_string());
            imports
        }
        _ => HashSet::new(),
    }
}

fn hashmap_inner_import(inner: &WrapperType) -> String {
    match inner {
        WrapperType::Struct(name) => format!("from . import {name}"),
        WrapperType::Enum(name) => format!("from .{name} import {name}"),
        WrapperType::String => "from .global_state import RustString".to_string(),
        WrapperType::Vec(inner) => format!(
            "{}\nfrom .vec_{} import {}\nfrom typing import List",
            hashmap_inner_import(inner),
            inner.name(),
            python_container_class_name(&WrapperType::Vec(inner.clone()))
        ),
        WrapperType::Option(inner) => format!(
            "{}\nfrom .option_{} import {}\nfrom typing import Optional",
            hashmap_inner_import(inner),
            inner.name(),
            python_container_class_name(&WrapperType::Option(inner.clone()))
        ),
        WrapperType::HashMap(key, value) => format!(
            "{}\n{}\nfrom .hashmap_{}_{} import {}\nfrom typing import Dict",
            hashmap_inner_import(key),
            hashmap_inner_import(value),
            key.name(),
            value.name(),
            python_container_class_name(&WrapperType::HashMap(key.clone(), value.clone()))
        ),
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

fn gen_hashmap_wrapper_python(key: &WrapperType, value: &WrapperType) -> String {
    let name = format!("hashmap_{}_{}", key.name(), value.name());
    let class_name = format!("{}HashMap{}", key.name(), value.name());
    let key_hint = type_hint_from_wrapper_type(key);
    let value_hint = type_hint_from_wrapper_type(value);
    let new_fn = format!("{EXPORTED_SYMBOLS_PREFIX}new_{name}");
    let drop_fn = format!("{EXPORTED_SYMBOLS_PREFIX}drop_{name}");
    let insert_fn = format!("{EXPORTED_SYMBOLS_PREFIX}insert_{name}");
    let len_fn = format!("{EXPORTED_SYMBOLS_PREFIX}len_{name}");
    let key_at_fn = format!("{EXPORTED_SYMBOLS_PREFIX}key_at_{name}");
    let value_at_fn = format!("{EXPORTED_SYMBOLS_PREFIX}value_at_{name}");
    let key_argtypes = python_hashmap_argtypes(key);
    let value_argtypes = python_hashmap_argtypes(value);
    let key_call = python_hashmap_insert_args(key, "key");
    let value_call = python_hashmap_insert_args(value, "value");
    let key_get_type = python_hashmap_get_value(key, key_at_fn.as_str());
    let value_get_type = python_hashmap_get_value(value, value_at_fn.as_str());
    let key_get_expr = python_hashmap_get_expr(key, key_at_fn.as_str());
    let value_get_expr = python_hashmap_get_expr(value, value_at_fn.as_str());
    let value_import = if matches!(key, WrapperType::String) || matches!(value, WrapperType::String)
    {
        "from .global_state import RustString\n"
    } else {
        ""
    };
    let nested_imports = [key, value]
        .iter()
        .map(|wrapper_type| hashmap_inner_import(wrapper_type))
        .flat_map(|imports| imports.lines().map(str::to_owned).collect::<Vec<_>>())
        .filter(|line| !line.is_empty())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"
from typing import Dict
import ctypes
from .global_state import get_ffi_lib
{value_import}
{nested_imports}

class {class_name}:
    def __init__(self, ptr):
        self._ptr = ptr

    def raw_ptr(self):
        return self._ptr

    def leak(self):
        ptr = self._ptr
        self._ptr = None
        return ptr

    @staticmethod
    def from_dict(values: Dict[{key_hint}, {value_hint}]):
        {PYTHON_LIB_GETTER_NAME}().{new_fn}.restype = ctypes.c_void_p
        {PYTHON_LIB_GETTER_NAME}().{insert_fn}.argtypes = [
            ctypes.c_void_p, {key_argtypes}, {value_argtypes}
        ]
        ptr = {PYTHON_LIB_GETTER_NAME}().{new_fn}()
        for key, value in values.items():
            {PYTHON_LIB_GETTER_NAME}().{insert_fn}(ptr, {key_call}, {value_call})
        return {class_name}(ptr)

    def to_dict(self) -> Dict[{key_hint}, {value_hint}]:
        {PYTHON_LIB_GETTER_NAME}().{len_fn}.argtypes = [ctypes.c_void_p]
        {PYTHON_LIB_GETTER_NAME}().{len_fn}.restype = ctypes.c_size_t
        {PYTHON_LIB_GETTER_NAME}().{key_at_fn}.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
        {PYTHON_LIB_GETTER_NAME}().{key_at_fn}.restype = ctypes.{key_get_type}
        {PYTHON_LIB_GETTER_NAME}().{value_at_fn}.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
        {PYTHON_LIB_GETTER_NAME}().{value_at_fn}.restype = ctypes.{value_get_type}
        result = {{}}
        for index in range({PYTHON_LIB_GETTER_NAME}().{len_fn}(self._ptr)):
            key = {key_get_expr}
            value = {value_get_expr}
            result[key] = value
        return result

    def __del__(self):
        if getattr(self, "_ptr", None) is not None:
            {PYTHON_LIB_GETTER_NAME}().{drop_fn}(self._ptr)
            self._ptr = None
"#
    )
}

fn python_hashmap_argtypes(wrapper_type: &WrapperType) -> String {
    match wrapper_type {
        WrapperType::String => "ctypes.c_char_p, ctypes.c_size_t".to_string(),
        _ => format!("ctypes.{}", c_type_from_wrapper_type(wrapper_type)),
    }
}

fn python_hashmap_insert_args(wrapper_type: &WrapperType, name: &str) -> String {
    match wrapper_type {
        WrapperType::String => {
            format!("ctypes.c_char_p({name}.encode(\"utf-8\")), len({name}.encode(\"utf-8\"))")
        }
        WrapperType::Bool => format!("ctypes.c_bool({name})"),
        WrapperType::Struct(_) => format!("{name}.raw_ptr()"),
        WrapperType::Enum(_) => format!("{name}.to_ffi()"),
        WrapperType::Vec(_) => format!(
            "({name}_wrapper := {}.from_list({name})).raw_ptr()",
            python_container_class_name(wrapper_type)
        ),
        WrapperType::Option(_) => format!(
            "({name}_wrapper := {}.from_python({name})).raw_ptr()",
            python_container_class_name(wrapper_type)
        ),
        WrapperType::HashMap(_, _) => format!(
            "({name}_wrapper := {}.from_dict({name})).raw_ptr()",
            python_container_class_name(wrapper_type)
        ),
        WrapperType::FloatingPointNumber(inner) if inner == "f32" => {
            format!("ctypes.c_float({name})")
        }
        WrapperType::FloatingPointNumber(_) => format!("ctypes.c_double({name})"),
        _ => name.to_string(),
    }
}

fn python_hashmap_get_value(wrapper_type: &WrapperType, _fn_name: &str) -> String {
    match wrapper_type {
        WrapperType::String => "c_void_p".to_string(),
        _ => c_type_from_wrapper_type(wrapper_type).to_string(),
    }
}

fn python_hashmap_get_expr(wrapper_type: &WrapperType, fn_name: &str) -> String {
    match wrapper_type {
        WrapperType::String => {
            format!("RustString({PYTHON_LIB_GETTER_NAME}().{fn_name}(self._ptr, index)).py_str()")
        }
        WrapperType::Bool => {
            format!("bool({PYTHON_LIB_GETTER_NAME}().{fn_name}(self._ptr, index))")
        }
        WrapperType::Struct(name) => {
            format!("{name}.{name}({PYTHON_LIB_GETTER_NAME}().{fn_name}(self._ptr, index))")
        }
        WrapperType::Enum(name) => {
            format!("{name}.from_ffi({PYTHON_LIB_GETTER_NAME}().{fn_name}(self._ptr, index))")
        }
        WrapperType::Vec(_) => format!(
            "{}({PYTHON_LIB_GETTER_NAME}().{fn_name}(self._ptr, index)).to_list()",
            python_container_class_name(wrapper_type)
        ),
        WrapperType::Option(_) => format!(
            "{}({PYTHON_LIB_GETTER_NAME}().{fn_name}(self._ptr, index)).to_python()",
            python_container_class_name(wrapper_type)
        ),
        WrapperType::HashMap(_, _) => format!(
            "{}({PYTHON_LIB_GETTER_NAME}().{fn_name}(self._ptr, index)).to_dict()",
            python_container_class_name(wrapper_type)
        ),
        _ => format!("{PYTHON_LIB_GETTER_NAME}().{fn_name}(self._ptr, index)"),
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
        WrapperType::HashMap(key, value) => format!(
            "Dict[{}, {}]",
            type_hint_from_wrapper_type(key),
            type_hint_from_wrapper_type(value)
        ),
        WrapperType::Enum(name) => name.to_string(),
        WrapperType::Result(inner) => type_hint_from_wrapper_type(inner),
        WrapperType::Option(inner) => format!("Optional[{}]", type_hint_from_wrapper_type(inner)),
        WrapperType::UnitExpr => "None".to_string(),
        WrapperType::Trait(name) => format!("Type[{name}]"),
    }
}

pub(crate) fn python_container_class_name(wrapper_type: &WrapperType) -> String {
    match wrapper_type {
        WrapperType::Vec(inner) => format!("{}Vec", inner.name()),
        WrapperType::Option(inner) => format!("{}Option", inner.name()),
        WrapperType::HashMap(key, value) => {
            format!("{}HashMap{}", key.name(), value.name())
        }
        _ => unreachable!("Only container types have Python wrapper classes"),
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
        WrapperType::HashMap(key, value) => format!(
            "Dict[{}, {}]",
            type_hint_from_wrapper_type(key),
            type_hint_from_wrapper_type(value)
        ),
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
        WrapperType::Vec(_) => {
            format!(
                "return {}(result).to_list()",
                python_container_class_name(wrapper)
            )
        }
        WrapperType::HashMap(_, _) => {
            format!(
                "return {}(result).to_dict()",
                python_container_class_name(wrapper)
            )
        }
        WrapperType::Enum(name) => {
            format!("return {}.{}.from_ffi(result)", name, name)
        }
        WrapperType::Struct(ty) => {
            format!("return {}.{}(result)", ty, ty)
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
        WrapperType::Option(_) => {
            format!(
                "return {}(result).to_python()",
                python_container_class_name(wrapper)
            )
        }
        WrapperType::Trait(trait_name) => {
            format!("return {trait_name}.{trait_name}Impl(result)")
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
        WrapperType::HashMap(_, _) => "c_void_p",
        WrapperType::Enum(_) => "c_int",
        WrapperType::UnitExpr => "c_void",
    }
}

pub(crate) fn arg_cast(wrapper_type: &WrapperType, arg_name: &str) -> String {
    match wrapper_type {
        WrapperType::IntegerNumber(_) | WrapperType::Bool => arg_name.to_string(),
        WrapperType::FloatingPointNumber(inner) if inner == "f32" => {
            format!("ctypes.c_float({arg_name})")
        }
        WrapperType::FloatingPointNumber(_) => format!("ctypes.c_double({arg_name})"),
        WrapperType::String => format!(r#"ctypes.c_char_p({arg_name}.encode("utf-8"))"#),
        WrapperType::Vec(_) => format!(
            "{}.from_list({arg_name})",
            python_container_class_name(wrapper_type)
        ),
        WrapperType::Option(_) => format!(
            "{}.from_python({arg_name})",
            python_container_class_name(wrapper_type)
        ),
        WrapperType::HashMap(_, _) => format!(
            "{}.from_dict({arg_name})",
            python_container_class_name(wrapper_type)
        ),
        WrapperType::Enum(_) => format!("{arg_name}.to_ffi()"),
        WrapperType::Struct(_) | WrapperType::Trait(_) => {
            format!("{arg_name}.raw_ptr()")
        }
        WrapperType::Result(_) => panic!("Result types are not supported as arguments"),
        WrapperType::UnitExpr => unreachable!(),
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
        | WrapperType::HashMap(_, _)
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
        | WrapperType::Option(_)
        | WrapperType::HashMap(_, _) => {
            format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_void_p")
        }
        WrapperType::UnitExpr => {
            format!("{PYTHON_LIB_GETTER_NAME}().{extern_fn_name}.restype = ctypes.c_void")
        }
    }
}
