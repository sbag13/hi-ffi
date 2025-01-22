use super::*;
use crate::wrapper::base::*;

use class_definition::*;
use function_definition::*;

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
                source: None, // TODO
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
