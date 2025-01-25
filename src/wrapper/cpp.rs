use super::*;
use crate::wrapper::base::*;

use class_definition::*;
use function_definition::*;
use quote::ToTokens;

pub mod class_definition;
pub mod function_definition;

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
}

fn map_return_type(return_wrapper: &Option<FunctionReturnWrapper>) -> ReturnTypes {
    match return_wrapper {
        Some(FunctionReturnWrapper {
            wrapper_type: FunctionReturnWrapperType::Primitive,
            return_type,
        }) => ReturnTypes {
            ext_return_type: return_type.to_token_stream().to_string(),
            return_type: return_type.to_token_stream().to_string(),
            return_cast: "    return result;".to_string(),
        },
        Some(FunctionReturnWrapper {
            wrapper_type: FunctionReturnWrapperType::String,
            ..
        }) => ReturnTypes {
            ext_return_type: "void*".to_string(),
            return_type: "std::string".to_string(),
            return_cast: "
    auto rust_str = RustString(result);
    return rust_str.to_string();"
                .to_string(),
        },
        None => ReturnTypes {
            ext_return_type: "void*".to_string(),
            return_type: "void".to_string(),
            return_cast: "".to_string(),
        },
    }
}
