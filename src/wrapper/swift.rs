use class_definition::{
    gen_class_methods_definition_from_impl_block, gen_class_methods_definition_from_struct,
    gen_method_declarations_from_impl_block, gen_method_declarations_from_struct,
};
use function_definition::{gen_function_definition, gen_function_header};

use super::*;

pub mod class_definition;
pub mod function_definition;

impl ReusableWrapper {
    pub fn swift(&self) -> String {
        match self {
            ReusableWrapper::Vec(inner) => gen_vec_wrapper_swift(inner),
        }
    }
}

pub fn gen_swift_vec_declarations(inner: &WrapperType) -> String {
    let inner_name = inner.name();
    let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{inner_name}_vec");
    let with_capacity_ext_name =
        format!("{EXPORTED_SYMBOLS_PREFIX}__with_capacity_{inner_name}_vec");
    let push_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__push_{inner_name}_vec");
    let len_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__len_{inner_name}_vec");
    let get_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__get_{inner_name}_vec");

    let push_args = match inner {
        WrapperType::IntegerNumber(inner) | WrapperType::FloatingPointNumber(inner) => {
            format!("{inner} value")
        }
        WrapperType::Bool => "bool value".to_string(),
        WrapperType::String => "const char* value, unsigned int len".to_string(),
        WrapperType::Struct(_) => "void* value".to_string(),
        WrapperType::Vec(_) => panic!("Vec of vecs not supported"),
    };

    let get_return_type = match inner {
        WrapperType::IntegerNumber(t) | WrapperType::FloatingPointNumber(t) => t.to_string(),
        WrapperType::Bool => "u8".to_string(),
        WrapperType::String => "void*".to_string(),
        WrapperType::Struct(_) => "void*".to_string(),
        WrapperType::Vec(_) => panic!("Vec of vecs not supported"),
    };

    format!(
        r#"
void* {drop_ext_name}(void* self);
void* {with_capacity_ext_name}(size_t capacity);
void {push_ext_fn_name}(void* self, {push_args});
size_t {len_ext_fn_name}(void* self);
{get_return_type} {get_ext_fn_name}(void* self, size_t index);"#
    )
}

fn gen_vec_wrapper_swift(inner: &WrapperType) -> String {
    let wrapper_name = format!("Rust{}Vec", inner.name());
    let inner_name = inner.name();
    let inner_swift_name = match inner {
        WrapperType::String => "String".to_string(),
        WrapperType::Struct(name) => name.to_string(),
        _ => inner_name.clone(),
    };

    let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{inner_name}_vec");
    let with_capacity_ext_name =
        format!("{EXPORTED_SYMBOLS_PREFIX}__with_capacity_{inner_name}_vec");
    let push_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__push_{inner_name}_vec");
    let len_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__len_{inner_name}_vec");
    let get_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__get_{inner_name}_vec");

    let _push_ext_receiver = match inner {
        WrapperType::IntegerNumber(inner) | WrapperType::FloatingPointNumber(inner) => {
            format!("value: {inner}")
        }
        WrapperType::Bool => "value: bool".to_string(),
        WrapperType::String => "value: UnsafePointer<Int8>?, _ len: usize".to_string(),
        WrapperType::Struct(_inner) => "value: UnsafeMutableRawPointer?".to_string(),
        WrapperType::Vec(_) => panic!("Vec of vecs not supported yet!"),
    };

    let _get_ext_return_type = match inner {
        WrapperType::IntegerNumber(t) | WrapperType::FloatingPointNumber(t) => t.to_string(),
        WrapperType::Bool => "bool".to_string(),
        WrapperType::String => "UnsafeMutableRawPointer?".to_string(),
        WrapperType::Struct(_) => "UnsafeMutableRawPointer?".to_string(),
        WrapperType::Vec(_) => panic!("Vec of vecs not supported yet!"),
    };

    let loop_expressions = match inner {
        WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) | WrapperType::Bool => {
            format!("            {push_ext_fn_name}(rust_vec_ptr, elem)")
        }
        WrapperType::String => {
            format!(
                "            elem.withCString {{ ptr in\n                {push_ext_fn_name}(rust_vec_ptr, ptr, UInt32(elem.utf8.count))\n            }}"
            )
        }
        WrapperType::Struct(_) => {
            format!("            {push_ext_fn_name}(rust_vec_ptr, elem.rawPtr())")
        }
        _ => panic!("Pushing to vec not supported"),
    };

    let elem_fetch = match inner {
        WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) => {
            format!("            let elem = {get_ext_fn_name}(self.rawPtr(), i)")
        }
        WrapperType::Bool => {
            format!("            let elem = {get_ext_fn_name}(self.rawPtr(), i) != 0")
        }
        WrapperType::String => {
            format!(
                "            let elem_ptr = {get_ext_fn_name}(self.rawPtr(), i)\n            let elem = RustString(elem_ptr!).to_string()"
            )
        }
        WrapperType::Struct(_) => {
            format!(
                "            let elem_ptr = {get_ext_fn_name}(self.rawPtr(), i)\n            let elem = {inner_swift_name}(elem_ptr!)"
            )
        }
        WrapperType::Vec(_) => unreachable!(),
    };

    format!(
        r#"
import Foundation

open class {wrapper_name}: Opaque {{
    
    public required init(_ _self: UnsafeMutableRawPointer) {{
        super.init(_self)
    }}

    public static func fromSwift(_ other: [{inner_swift_name}]) -> {wrapper_name} {{
        let rust_vec_ptr = {with_capacity_ext_name}(other.count)
        for elem in other {{
{loop_expressions}
        }}
        return {wrapper_name}(rust_vec_ptr!)
    }}

    public func toSwift() -> [{inner_swift_name}] {{
        let len = {len_ext_fn_name}(self.rawPtr())
        var result: [{inner_swift_name}] = []
        result.reserveCapacity(Int(len))
        for i in 0..<len {{
{elem_fetch}
            result.append(elem)
        }}
        return result
    }}

    deinit {{
        {drop_ext_name}(self.rawPtr())
    }}
}}
"#
    )
}

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
