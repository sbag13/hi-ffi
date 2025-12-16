use super::*;
use crate::wrapper::base::*;

use class_definition::*;
use function_definition::*;
use quote::ToTokens;

pub mod class_definition;
pub mod function_definition;

impl ReusableWrapper {
    pub fn cpp(&self) -> String {
        match self {
            ReusableWrapper::Vec(inner) => gen_vec_wrapper_cpp(inner),
        }
    }
}

// TODO split to functions
fn gen_vec_wrapper_cpp(inner: &WrapperType) -> String {
    let wrapper_name = format!("Rust{}Vec", inner.name());
    let inner_name = inner.name();
    let inner_cpp_name = match inner {
        WrapperType::String => "std::string".to_string(),
        _ => inner_name.clone(),
    };
    let push_ext_receiver = match inner {
        WrapperType::IntegerNumber(inner) | WrapperType::FloatingPointNumber(inner) => {
            format!("{inner} value")
        }
        WrapperType::Struct(inner) => format!("void* {inner}"),
        WrapperType::Vec(_) => unimplemented!("CPP: vec of vecs unimplemented!"),
        WrapperType::Bool => "bool value".to_string(),
        WrapperType::String => "const char* value".to_string(), // 2nd arg, len, is ignored for now
    };
    let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{inner_name}_vec");
    let with_capacity_ext_name =
        format!("{EXPORTED_SYMBOLS_PREFIX}__with_capacity_{inner_name}_vec");
    let push_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__push_{inner_name}_vec");

    let loop_expressions = match inner {
        WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) | WrapperType::Bool => {
            format!("           {push_ext_fn_name}(rust_vec_ptr, elem);")
        }
        WrapperType::String => {
            format!("            {push_ext_fn_name}(rust_vec_ptr, elem.data());")
        }
        WrapperType::Struct(_) => {
            format!("            {push_ext_fn_name}(rust_vec_ptr, elem.self_ptr());")
        }
        _ => panic!("Pushing to vec not supported"),
    };

    let includes = match inner {
        WrapperType::Struct(struct_name) => format!(r#"#include "{struct_name}.h""#),
        _ => String::new(),
    };

    format!(
        r#"
#ifndef {wrapper_name}__def
#define {wrapper_name}__def

#include "base.h"
#include <vector>
{includes}

extern "C" {{
    void* {drop_ext_name}(void* self);
    void* {with_capacity_ext_name}(size_t capacity);
    void* {push_ext_fn_name}(void* self, {push_ext_receiver});
}}

class {wrapper_name} {{
    void* self = nullptr;
public:
    {wrapper_name}(void* self) : self(self) {{}}

    void* raw_ptr() {{
        return this->self;
    }}

    virtual ~{wrapper_name}() {{
        if (this->self != nullptr)
            {drop_ext_name}(this->self);
    }}

    static {wrapper_name} from_std(const std::vector<{inner_cpp_name}>& other) {{
        void* rust_vec_ptr = {with_capacity_ext_name}(other.size());
        for (const auto& elem : other) {{
{loop_expressions}
        }}
        return {wrapper_name}(rust_vec_ptr);
    }}
}};

#endif
"#
    )
}

pub enum CppHeader {
    Class(ClassHeaderParts),
    Function(String),
}

pub struct CppFiles {
    pub header: CppHeader,
    pub source: Option<String>,
}

impl Wrapper {
    pub fn cpp(&self) -> CppFiles {
        match &self.parsed {
            ParsedWrapper::Struct(struct_wrapper) => CppFiles {
                header: CppHeader::Class(gen_class_definition_parts_from_struct(struct_wrapper)),
                source: None,
            },
            ParsedWrapper::Function(function_wrapper) => CppFiles {
                header: CppHeader::Function(gen_function_declaration(function_wrapper)),
                source: Some(gen_function_definition(function_wrapper)),
            },
            ParsedWrapper::ImplBlock(impl_block_wrapper) => CppFiles {
                header: CppHeader::Class(gen_class_definition_parts_from_impl_block(
                    impl_block_wrapper,
                )),
                source: None,
            },
        }
    }
}

struct Methods {
    getter: Option<Method>,
    setter: Option<Method>,
}

struct Method {
    definition: String,
    extern_fn: String,
    include: String,
}

struct ReturnTypes {
    ext_return_type: String,
    return_type: String,
    return_cast: String,
    return_type_includes: String,
}

fn map_return_type(return_wrapper: &Option<FunctionReturnWrapper>) -> ReturnTypes {
    match return_wrapper {
        Some(FunctionReturnWrapper {
            wrapper_type:
                WrapperType::IntegerNumber(_) | WrapperType::Bool | WrapperType::FloatingPointNumber(_),
            return_type,
        }) => ReturnTypes {
            ext_return_type: return_type.to_token_stream().to_string(),
            return_type: return_type.to_token_stream().to_string(),
            return_cast: "return result;".to_string(),
            return_type_includes: String::new(),
        },
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::String,
            ..
        }) => ReturnTypes {
            ext_return_type: "void*".to_string(),
            return_type: "std::string".to_string(),
            return_cast: "
auto rust_str = RustString(result);
return rust_str.to_string();"
                .to_string(),
            return_type_includes: String::new(),
        },
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Struct(_),
            return_type,
        }) => {
            let struct_type = return_type.to_token_stream().to_string();
            ReturnTypes {
                ext_return_type: "void*".to_string(),
                return_type: struct_type.clone(),
                return_cast: format!(
                    "
return {}(result);",
                    struct_type
                ),
                return_type_includes: format!("#include \"{struct_type}.h\""),
            }
        }
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Vec(_),
            ..
        }) => unimplemented!("Vec of vecs not supported yet as a cpp return type!"),
        None => ReturnTypes {
            ext_return_type: "void*".to_string(),
            return_type: "void".to_string(),
            return_cast: "".to_string(),
            return_type_includes: String::new(),
        },
    }
}
