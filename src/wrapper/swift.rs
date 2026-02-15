use class_definition::{
    gen_class_methods_definition_from_impl_block, gen_class_methods_definition_from_struct,
    gen_method_declarations_from_impl_block, gen_method_declarations_from_struct,
};
use function_definition::{gen_function_definition, gen_function_header};

use crate::{prepend_each_line_with_n_tabs, wrapper::swift::enum_definition::gen_enum_code};

use super::*;

pub mod class_definition;
pub mod enum_definition;
pub mod function_definition;

// Helper functions for common type conversions and patterns
fn get_c_return_type(wrapper_type: &WrapperType) -> String {
    match wrapper_type {
        WrapperType::IntegerNumber(t) | WrapperType::FloatingPointNumber(t) => t.to_string(),
        WrapperType::Bool => "u8".to_string(),
        WrapperType::String => "void*".to_string(),
        WrapperType::Struct(_) => "void*".to_string(),
        WrapperType::Vec(_) => "void*".to_string(),
        WrapperType::Result(_) => panic!("Result of results not supported"),
        WrapperType::Option(_) => panic!("Option in result not supported"),
        WrapperType::Enum(name) => format!("enum {}", name),
        WrapperType::UnitExpr => "void".to_string(),
    }
}

fn get_swift_type_name(wrapper_type: &WrapperType) -> String {
    match wrapper_type {
        WrapperType::IntegerNumber(t) | WrapperType::FloatingPointNumber(t) => t.to_string(),
        WrapperType::Bool => "bool".to_string(),
        WrapperType::String => "String".to_string(),
        WrapperType::Struct(name) => name.to_string(),
        WrapperType::Vec(vec_inner) => format!("[{}]", vec_inner.name()),
        WrapperType::Result(_) => panic!("Result of results not supported"),
        WrapperType::Option(_) => panic!("Option in result not supported"),
        WrapperType::Enum(name) => name.to_string(),
        WrapperType::UnitExpr => "Void".to_string(),
    }
}

fn get_swift_wrapper_type_name(wrapper_type: &WrapperType) -> String {
    match wrapper_type {
        WrapperType::String => "String".to_string(),
        WrapperType::Struct(name) => name.to_string(),
        _ => wrapper_type.name(),
    }
}

fn generate_unwrap_cast(wrapper_type: &WrapperType, ext_call: &str) -> String {
    match wrapper_type {
        WrapperType::String => format!("let result = RustString({ext_call}).to_string();"),
        WrapperType::Struct(struct_name) => {
            format!("let result = {struct_name}({ext_call});")
        }
        WrapperType::Vec(vec_inner) => {
            let vec_inner_name = vec_inner.name();
            format!("let result = Rust{vec_inner_name}Vec({ext_call}).toSwift();")
        }
        WrapperType::Bool => format!("let result = {ext_call} != 0;"),
        WrapperType::Enum(inner) => {
            format!(
                "let raw_result = {ext_call}\n\
                let result = {inner}(rawValue: Int32(raw_result.rawValue))!;"
            )
        }
        WrapperType::UnitExpr => format!("{ext_call};"),
        _ => format!("let result = {ext_call};"),
    }
}

impl ReusableWrapper {
    pub fn swift(&self) -> String {
        match self {
            ReusableWrapper::Vec(inner) => gen_vec_wrapper_swift(inner),
            ReusableWrapper::Result(inner) => gen_result_wrapper_swift(inner),
            ReusableWrapper::Option(inner) => gen_option_wrapper_swift(inner),
        }
    }
}

pub fn gen_swift_result_declarations(inner: &WrapperType) -> String {
    let inner_name = inner.name();
    let unwrap_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__unwrap_{inner_name}_result");
    let unwrap_err_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__unwrap_err_{inner_name}_result");
    let drop_err_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{inner_name}_result");
    let is_err_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__is_err_{inner_name}_result");

    let unwrap_return_type = get_c_return_type(inner);

    format!(
        r#"
{unwrap_return_type} {unwrap_ext_name}(void* self);
void* {unwrap_err_ext_name}(void* self);
void {drop_err_ext_name}(void* self);
u8 {is_err_ext_name}(void* self);
"#
    )
}

pub fn gen_swift_option_declarations(inner: &WrapperType) -> String {
    let inner_name = inner.name();
    let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{inner_name}_option");
    let unwrap_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__unwrap_{inner_name}_option");
    let is_some_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__is_some_{inner_name}_option");
    let some_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__some_{inner_name}_option");
    let none_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__none_{inner_name}_option");

    let unwrap_return_type = get_c_return_type(inner);

    let some_arg_type = match inner {
        WrapperType::String => "const char*".to_string(),
        _ => unwrap_return_type.clone(),
    };

    format!(
        r#"
void {drop_ext_name}(void* self);
{unwrap_return_type} {unwrap_ext_name}(void* self);
u8 {is_some_ext_name}(void* self);
void* {some_ext_name}({some_arg_type} val);
void* {none_ext_name}();
"#
    )
}

pub fn gen_result_wrapper_swift(inner: &WrapperType) -> String {
    let inner_name = inner.name();

    let unwrap_ret_type = get_swift_type_name(inner);

    let unwrap_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__unwrap_{}_result", inner_name);
    let unwrap_ext_call = format!("{unwrap_ext_name}(self.rawPtr())");
    let unwrap_val_cast = generate_unwrap_cast(inner, &unwrap_ext_call);

    let unwrap_return_expr = match inner {
        WrapperType::UnitExpr => "return",
        _ => "return result",
    };

    let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{}_result", inner_name);

    let unwrap_val_cast = prepend_each_line_with_n_tabs(&unwrap_val_cast, 2);

    format!(
        r#"
import Foundation

open class Rust{inner_name}Result: Opaque {{
    public required init(_ _self: UnsafeMutableRawPointer) {{
        super.init(_self)
    }}

    func isErr() -> Bool {{
        return {EXPORTED_SYMBOLS_PREFIX}__is_err_{inner_name}_result(self.rawPtr()) != 0
    }}

    func unwrap() -> {unwrap_ret_type} {{
{unwrap_val_cast}
        {unwrap_return_expr};
    }}

    func unwrapErr() -> UnsafeMutableRawPointer {{
        return {EXPORTED_SYMBOLS_PREFIX}__unwrap_err_{inner_name}_result(self.rawPtr())
    }}

    deinit {{
        {drop_ext_name}(self.rawPtr())
    }}
}}
"#
    )
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
        WrapperType::Result(_) => panic!("Vec of results not supported"),
        WrapperType::Option(_) => panic!("Vec of options not supported"),
        WrapperType::Enum(name) => format!("enum {} value", name),
        WrapperType::UnitExpr => unreachable!(),
    };

    let get_return_type = match inner {
        WrapperType::IntegerNumber(t) | WrapperType::FloatingPointNumber(t) => t.to_string(),
        WrapperType::Bool => "u8".to_string(),
        WrapperType::String => "void*".to_string(),
        WrapperType::Struct(_) => "void*".to_string(),
        WrapperType::Vec(_) => panic!("Vec of vecs not supported"),
        WrapperType::Result(_) => panic!("Vec of results not supported"),
        WrapperType::Option(_) => panic!("Vec of options not supported"),
        WrapperType::Enum(name) => format!("enum {}", name),
        WrapperType::UnitExpr => unreachable!(),
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
    let inner_swift_name = get_swift_wrapper_type_name(inner);

    let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{inner_name}_vec");
    let with_capacity_ext_name =
        format!("{EXPORTED_SYMBOLS_PREFIX}__with_capacity_{inner_name}_vec");
    let push_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__push_{inner_name}_vec");
    let len_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__len_{inner_name}_vec");
    let get_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__get_{inner_name}_vec");

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
        WrapperType::Enum(_) => {
            format!(
                "            {push_ext_fn_name}(rust_vec_ptr, CFfiModule.{inner_swift_name}(rawValue: UInt32(elem.rawValue)))"
            )
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
        WrapperType::Enum(_) => {
            format!(
                "            let c_elem = {get_ext_fn_name}(self.rawPtr(), i)\n            let elem = {inner_swift_name}(rawValue: Int32(c_elem.rawValue))!"
            )
        }
        WrapperType::Vec(_) => unreachable!(),
        WrapperType::Result(_) => panic!("Vec of results not supported"),
        WrapperType::Option(_) => panic!("Vec of options not supported"),
        WrapperType::UnitExpr => unreachable!(),
    };

    format!(
        r#"
import Foundation

open class {wrapper_name}: Opaque {{
    private var leaked = false

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

    public func leak() {{
        leaked = true
    }}

    deinit {{
        if !leaked {{
            {drop_ext_name}(self.rawPtr())
        }}
    }}
}}
"#
    )
}

fn gen_option_wrapper_swift(inner: &WrapperType) -> String {
    let wrapper_name = format!("Rust{}Option", inner.name());
    let inner_name = inner.name();
    let inner_swift_name = get_swift_wrapper_type_name(inner);

    let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{inner_name}_option");
    let unwrap_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__unwrap_{inner_name}_option");
    let is_some_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__is_some_{inner_name}_option");
    let some_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__some_{inner_name}_option");
    let none_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__none_{inner_name}_option");

    let unwrap_ext_call = format!("{unwrap_ext_name}(self.rawPtr())");
    let unwrap_val_cast = generate_unwrap_cast(inner, &unwrap_ext_call);

    let unwrap_return_expr = match inner {
        WrapperType::UnitExpr => "return",
        _ => "return result",
    };

    let from_swift_cast = match inner {
        WrapperType::String => {
            format!(
                "return some_val.withCString {{ ptr in\n                return {wrapper_name}({some_ext_name}(ptr))\n            }}"
            )
        }
        WrapperType::Struct(_) => {
            format!("return {wrapper_name}({some_ext_name}(some_val.rawPtr()))")
        }
        WrapperType::Bool => {
            format!("return {wrapper_name}({some_ext_name}(some_val ? 1 : 0))")
        }
        WrapperType::Enum(_) => {
            format!(
                "return {wrapper_name}({some_ext_name}(CFfiModule.{inner_swift_name}(rawValue: UInt32(some_val.rawValue))))"
            )
        }
        _ => format!("return {wrapper_name}({some_ext_name}(some_val))"),
    };

    let unwrap_val_cast = prepend_each_line_with_n_tabs(&unwrap_val_cast, 2);

    format!(
        r#"
import Foundation

open class {wrapper_name}: Opaque {{
    public required init(_ _self: UnsafeMutableRawPointer) {{
        super.init(_self)
    }}

    public static func fromSwift(_ other: {inner_swift_name}?) -> {wrapper_name} {{
        if let some_val = other {{
            {from_swift_cast}
        }} else {{
            return {wrapper_name}({none_ext_name}())
        }}
    }}

    public func toSwift() -> {inner_swift_name}? {{
        if {is_some_ext_name}(self.rawPtr()) != 0 {{
{unwrap_val_cast}
            {unwrap_return_expr}
        }} else {{
            return nil
        }}
    }}

    func isSome() -> Bool {{
        return {is_some_ext_name}(self.rawPtr()) != 0
    }}

    func unwrap() -> {inner_swift_name} {{
{unwrap_val_cast}
        {unwrap_return_expr}
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
    Enum { header: String, source: String },
}

impl SwiftCode {
    pub fn header(&self) -> String {
        match self {
            SwiftCode::Class { header, .. } => header.to_owned(),
            SwiftCode::Function { header, .. } => header.to_owned(),
            SwiftCode::Enum { header, .. } => header.to_owned(),
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
            ParsedWrapper::Enum(enum_wrapper) => gen_enum_code(enum_wrapper),
        }
    }
}
