use class_definition::{
    gen_class_methods_definition_from_impl_block, gen_class_methods_definition_from_struct,
    gen_method_declarations_from_impl_block, gen_method_declarations_from_struct,
};
use function_definition::{gen_function_definition, gen_function_header};

use super::*;

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
                header: gen_method_declarations_from_struct(struct_wrapper),
                source: gen_class_methods_definition_from_struct(struct_wrapper),
            },
            ParsedWrapper::Function(function_wrapper) => {
                let function_definition = gen_function_definition(function_wrapper);
                let source = format!(
                    r#"@_exported import CFfiModule
{function_definition}"#
                );
                SwiftCode::Function {
                    header: gen_function_header(
                        &function_wrapper.extern_function_name,
                        &function_wrapper.args_wrappers,
                        &function_wrapper.return_wrapper,
                    ),
                    source,
                }
            }
            ParsedWrapper::ImplBlock(impl_block_wrapper) => SwiftCode::Class {
                header: gen_method_declarations_from_impl_block(impl_block_wrapper),
                source: gen_class_methods_definition_from_impl_block(impl_block_wrapper),
            },
        }
    }
}
