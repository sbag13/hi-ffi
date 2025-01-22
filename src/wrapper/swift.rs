use super::*;

use class_definition::*;
use function_definition::*;

pub mod class_definition;
pub mod function_definition;

pub enum SwiftCode {
    Class { header: String, source: String },
    Function { header: String, source: String },
}

impl SwiftCode {
    pub fn header(&self) -> String {
        match self {
            SwiftCode::Class { header, .. } => header.to_owned(),
            SwiftCode::Function { header, .. } => header.to_owned(),
        }
    }
}

impl Wrapper {
    pub fn swift(&self) -> SwiftCode {
        match &self.parsed {
            ParsedWrapper::Struct(struct_wrapper) => SwiftCode::Class {
                header: gen_class_header(struct_wrapper),
                source: gen_class_methods_from_struct(struct_wrapper),
            },
            ParsedWrapper::Function(function_wrapper) => SwiftCode::Function {
                header: gen_function_header(function_wrapper),
                source: gen_function_definition(function_wrapper),
            },
            ParsedWrapper::ImplBlock(_impl_block_wrapper) => SwiftCode::Class {
                header: "".to_string(), // TODO
                source: "".to_string(), // TODO
            },
        }
    }
}
