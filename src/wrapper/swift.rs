use super::*;

use class_definition::*;
use function_definition::*;

pub mod class_definition;
pub mod function_definition;

pub struct SwiftFiles {
    pub header: String,
    pub source: String,
}

impl Wrapper {
    pub fn swift(&self) -> SwiftFiles {
        match &self.parsed {
            ParsedWrapper::Struct(struct_wrapper) => SwiftFiles {
                header: gen_class_header(struct_wrapper),
                source: gen_class_definition(struct_wrapper),
            },
            ParsedWrapper::Function(function_wrapper) => SwiftFiles {
                header: gen_function_header(function_wrapper),
                source: gen_function_definition(function_wrapper),
            },
        }
    }
}
