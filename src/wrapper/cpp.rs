use super::*;
use crate::wrapper::base::*;

use class_definition::*;
use function_definition::*;

pub mod class_definition;
pub mod function_definition;

pub struct CppFiles {
    pub header: String,
    pub source: Option<String>,
}

impl Wrapper {
    pub fn cpp(&self) -> CppFiles {
        match &self.parsed {
            ParsedWrapper::Struct(struct_wrapper) => CppFiles {
                header: gen_class_definition(struct_wrapper),
                source: None,
            },
            ParsedWrapper::Function(function_wrapper) => CppFiles {
                header: gen_function_declaration(function_wrapper),
                source: Some(gen_function_definition(function_wrapper)),
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
