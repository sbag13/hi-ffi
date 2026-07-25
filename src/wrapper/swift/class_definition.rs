use std::fmt::Display;
use std::ops::Deref;

use crate::wrapper::base::{SLICE_DROP_FN_NAME, SLICE_GET_LEN_FN_NAME, SLICE_GET_PTR_FN_NAME};
use crate::wrapper::swift::function_definition::{
    ReturnTypes, compose_function_definition, map_args, map_header_declaration_args,
    map_return_type,
};
use crate::wrapper::{
    DefaultConstructor, FieldWrapper, Getter, ImplBlockWrapper, PartialEqImpl, Setter,
    StructWrapper, WrapperType,
};
use quote::ToTokens;

use super::impl_block_wrapper::MethodWrapper;

pub const METHOD_DEFINITIONS_MARKER: &str = "// class method definitions";

pub fn gen_method_declarations_from_struct(struct_wrapper: &StructWrapper) -> String {
    let destructor_extern_fn = &struct_wrapper.drop_ext_fn_name;
    let getters_and_setters = gen_getters_and_setters_externs(struct_wrapper);
    let default_constructor = struct_wrapper
        .default_constructor
        .as_ref()
        .map(gen_default_constructor_ext)
        .unwrap_or_default();
    let partial_eq_ext = struct_wrapper
        .partial_eq
        .as_ref()
        .map(gen_partial_eq_ext)
        .unwrap_or_default();

    format!(
        r#"
void {destructor_extern_fn}(void*);
{getters_and_setters}
{default_constructor}
{partial_eq_ext}
"#
    )
}

pub fn gen_method_declarations_from_impl_block(impl_block_wrapper: &ImplBlockWrapper) -> String {
    let mut declarations: Vec<String> = impl_block_wrapper
        .methods
        .iter()
        .filter(|&method| method.public)
        .map(gen_method_header)
        .collect();

    // If this impl block provides a Default implementation, add the extern fn declaration.
    if let Some(dc) = &impl_block_wrapper.default_constructor {
        declarations.push(gen_default_constructor_ext(dc));
    }

    // If this impl block provides a PartialEq implementation, add the extern fn declaration.
    if let Some(partial_eq) = &impl_block_wrapper.partial_eq {
        declarations.push(gen_partial_eq_ext(partial_eq));
    }

    declarations.join("\n")
}

pub fn gen_method_header(method: &MethodWrapper) -> String {
    let swift_args = map_header_declaration_args(&method.args);
    let swift_args = if method.is_static {
        swift_args
    } else if swift_args.is_empty() {
        "void* self".to_string()
    } else {
        format!("void* self, {}", swift_args)
    };

    let ReturnTypes {
        cpp_return_type, ..
    } = map_return_type(&method.return_wrapper);
    let extern_fn_name = &method.extern_function_name;

    format!(r#"{cpp_return_type} {extern_fn_name}({swift_args});"#)
}

pub fn gen_class_methods_definition_from_impl_block(
    impl_block_wrapper: &ImplBlockWrapper,
) -> String {
    let mut definitions: Vec<String> = impl_block_wrapper
        .methods
        .iter()
        .filter(|&method| method.public)
        .map(|method| {
            gen_method_definition(method)
                .lines()
                .filter_map(|line| {
                    if line.is_empty() {
                        None
                    } else {
                        Some(format!("    {}", line))
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        })
        .collect();

    // If this impl block provides a Default implementation, add the convenience init.
    if let Some(dc) = &impl_block_wrapper.default_constructor {
        definitions.push(gen_default_constructor(dc));
    }

    // If this impl block provides a PartialEq implementation, add the equality operator.
    if let Some(partial_eq) = &impl_block_wrapper.partial_eq {
        definitions.push(gen_partial_eq_impl(
            partial_eq,
            &impl_block_wrapper.struct_name,
        ));
    }

    definitions.join("\n")
}

fn gen_method_definition(method: &MethodWrapper) -> String {
    let mut mapped_args = map_args(method.args.iter());
    if !method.is_static {
        if method.args.is_empty() {
            mapped_args.args_names = "self.rawPtr()".to_string();
        } else {
            mapped_args.args_names = format!("self.rawPtr(), {}", mapped_args.args_names);
        }
    }

    let return_types = map_return_type(&method.return_wrapper);

    compose_function_definition(
        &method.name,
        &method.extern_function_name,
        mapped_args,
        return_types,
        method.is_static,
    )
}

fn gen_default_constructor_ext(dc: &DefaultConstructor) -> String {
    let default_constructor_ext_fn_name = &dc.extern_fn_name;

    format!(
        r#"
void* {default_constructor_ext_fn_name}();
"#
    )
}

fn gen_partial_eq_ext(partial_eq: &PartialEqImpl) -> String {
    let extern_fn_name = &partial_eq.extern_fn_name;

    format!(
        r#"
bool {extern_fn_name}(void* lhs, void* rhs);
"#
    )
}

fn gen_getters_and_setters_externs(struct_wrapper: &StructWrapper) -> String {
    struct_wrapper
        .fields
        .iter()
        .map(gen_getter_and_setter_externs)
        .collect()
}

fn gen_getter_and_setter_externs(field: &FieldWrapper) -> String {
    let field_type = &field.field_type.to_token_stream().to_string();
    let (getter, setter) = match field {
        FieldWrapper {
            wrapper_type:
                WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) | WrapperType::Bool,
            setter,
            getter,
            ..
        } => (
            getter
                .as_ref()
                .map(|g| map_primitive_getter_as_extern_fn(g, field_type)),
            setter
                .as_ref()
                .map(|g| map_primitive_setter_as_extern_fn(g, field_type)),
        ),

        FieldWrapper {
            wrapper_type: WrapperType::Enum(_),
            setter,
            getter,
            ..
        } => (
            getter
                .as_ref()
                .map(|g| map_enum_getter_as_extern_fn(g, field_type)),
            setter
                .as_ref()
                .map(|g| map_enum_setter_as_extern_fn(g, field_type)),
        ),

        FieldWrapper {
            wrapper_type: WrapperType::Struct(_),
            getter,
            setter,
            ..
        } => {
            let getter_code = getter.as_ref().map(|g| {
                format!(
                    "void* {extern_fn_name}(void*);",
                    extern_fn_name = g.extern_fn_name
                )
            });
            let setter_code = setter.as_ref().map(|s| {
                format!(
                    "void {extern_fn_name}(void*, void*);",
                    extern_fn_name = s.extern_fn_name
                )
            });
            (getter_code, setter_code)
        }

        FieldWrapper {
            wrapper_type: WrapperType::String,
            setter,
            getter,
            ..
        } => (
            getter.as_ref().map(map_string_getter_as_extern_fn),
            setter.as_ref().map(map_string_setter_as_extern_fn),
        ),

        FieldWrapper {
            wrapper_type: WrapperType::Vec(_),
            getter,
            setter,
            ..
        } => {
            let getter_code = getter.as_ref().map(|g| {
                format!(
                    "void* {extern_fn_name}(void*);",
                    extern_fn_name = g.extern_fn_name
                )
            });
            let setter_code = setter.as_ref().map(|s| {
                format!(
                    "void {extern_fn_name}(void*, void*);",
                    extern_fn_name = s.extern_fn_name
                )
            });
            (getter_code, setter_code)
        }

        FieldWrapper {
            wrapper_type: WrapperType::Option(_),
            getter,
            setter,
            ..
        } => {
            let getter_code = getter.as_ref().map(|g| {
                format!(
                    "void* {extern_fn_name}(void*);",
                    extern_fn_name = g.extern_fn_name
                )
            });
            let setter_code = setter.as_ref().map(|s| {
                format!(
                    "void {extern_fn_name}(void*, void*);",
                    extern_fn_name = s.extern_fn_name
                )
            });
            (getter_code, setter_code)
        }

        FieldWrapper {
            wrapper_type: WrapperType::Result(_) | WrapperType::Trait(_) | WrapperType::UnitExpr,
            ..
        } => panic!("Unsupported wrapper type for struct field"),
    };

    match (getter, setter) {
        (Some(getter), Some(setter)) => format!("\n{}\n{}", getter, setter),
        (Some(getter), None) => format!("\n{}", getter),
        (None, Some(setter)) => format!("\n{}", setter),
        (None, None) => String::new(),
    }
}

fn map_string_getter_as_extern_fn(Getter { extern_fn_name, .. }: &Getter) -> String {
    format!("void* {extern_fn_name}(void*);")
}

fn map_string_setter_as_extern_fn(Setter { extern_fn_name, .. }: &Setter) -> String {
    format!("void {extern_fn_name}(void*, const char*, unsigned int);")
}

fn map_primitive_getter_as_extern_fn(
    Getter { extern_fn_name, .. }: &Getter,
    field_type: impl Display,
) -> String {
    format!("{field_type} {extern_fn_name}(void*);")
}

fn map_enum_getter_as_extern_fn(
    Getter { extern_fn_name, .. }: &Getter,
    field_type: impl Display,
) -> String {
    format!("enum {field_type} {extern_fn_name}(void*);")
}

fn map_enum_setter_as_extern_fn(
    Setter { extern_fn_name, .. }: &Setter,
    field_type: impl Display,
) -> String {
    format!("void {extern_fn_name}(void*, enum {field_type});")
}

fn map_primitive_setter_as_extern_fn(
    Setter { extern_fn_name, .. }: &Setter,
    field_type: impl Display,
) -> String {
    format!("void {extern_fn_name}(void*, {field_type});")
}

pub fn gen_empty_class_definition(class_name: impl Display) -> String {
    format!(
        r#"
public class {class_name}: Opaque {{
    {METHOD_DEFINITIONS_MARKER}
}}
"#
    )
}

pub fn gen_class_methods_definition_from_struct(struct_wrapper: &StructWrapper) -> String {
    let destructor_extern_fn = &struct_wrapper.drop_ext_fn_name;
    let props = gen_props(struct_wrapper);
    let default_constructor = struct_wrapper
        .default_constructor
        .as_ref()
        .map(gen_default_constructor)
        .unwrap_or_default();
    let partial_eq_impl = struct_wrapper
        .partial_eq
        .as_ref()
        .map(|partial_eq| gen_partial_eq_impl(partial_eq, &struct_wrapper.name))
        .unwrap_or_default();

    format!(
        r#"
    deinit {{
        if self._self != nil {{
            {destructor_extern_fn}(self.rawPtr())
        }}
    }}
{default_constructor}
{props}
{partial_eq_impl}
"#
    )
}

fn gen_default_constructor(dc: &DefaultConstructor) -> String {
    let default_constructor_ext_fn_name = &dc.extern_fn_name;

    format!(
        r#"
    public convenience init() {{
        self.init({default_constructor_ext_fn_name}())
    }}
"#
    )
}

fn gen_partial_eq_impl(partial_eq: &PartialEqImpl, class_name: impl Display) -> String {
    let extern_fn_name = &partial_eq.extern_fn_name;

    format!(
        r#"
    public static func == (lhs: {class_name}, rhs: {class_name}) -> Bool {{
        return {extern_fn_name}(lhs.rawPtr(), rhs.rawPtr())
    }}
    public static func != (lhs: {class_name}, rhs: {class_name}) -> Bool {{
        return !{extern_fn_name}(lhs.rawPtr(), rhs.rawPtr())
    }}
"#,
    )
}

fn gen_props(struct_wrapper: &StructWrapper) -> String {
    struct_wrapper.fields.iter().map(gen_property).collect()
}

fn gen_property(field: &FieldWrapper) -> String {
    let (getter, setter, swift_field_type) = match field {
        FieldWrapper {
            wrapper_type:
                WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) | WrapperType::Bool,
            setter,
            getter,
            ..
        } => (
            getter.as_ref().map(map_primitive_getter),
            setter.as_ref().map(map_primitive_setter),
            field.field_type.to_token_stream().to_string(),
        ),

        FieldWrapper {
            wrapper_type: WrapperType::Enum(e_name),
            setter,
            getter,
            ..
        } => (
            getter.as_ref().map(|g| map_enum_getter(g, e_name)),
            setter.as_ref().map(|s| map_enum_setter(s, e_name)),
            field.field_type.to_token_stream().to_string(),
        ),

        FieldWrapper {
            wrapper_type: WrapperType::Struct(s_ty),
            setter,
            getter,
            field_type,
            ..
        } => (
            getter
                .as_ref()
                .map(|g| map_custom_getter(g, field_type.to_token_stream())),
            setter.as_ref().map(map_custom_setter),
            s_ty.clone(),
        ),

        FieldWrapper {
            wrapper_type: WrapperType::String,
            setter,
            getter,
            ..
        } => (
            getter.as_ref().map(map_string_getter),
            setter.as_ref().map(map_string_setter),
            field.field_type.to_token_stream().to_string(),
        ),

        FieldWrapper {
            wrapper_type: WrapperType::Vec(inner),
            setter,
            getter,
            ..
        } => {
            let swift_inner_type = match inner.deref() {
                WrapperType::IntegerNumber(name)
                | WrapperType::FloatingPointNumber(name)
                | WrapperType::Struct(name)
                | WrapperType::Enum(name) => name.clone(),
                WrapperType::Bool => "bool".to_string(),
                WrapperType::String => "String".to_string(),
                WrapperType::Vec(_) => "Array".to_string(), // Nested vectors not supported yet
                WrapperType::Result(_) => panic!("Vec of results not supported as property"),
                WrapperType::Option(_) => panic!("Option in vec not supported as property"),
                WrapperType::UnitExpr => panic!("Empty expression cannot be a swift property"),
                WrapperType::Trait(_) => panic!("Trait not supported as property"),
            };
            (
                getter
                    .as_ref()
                    .map(|g| map_vec_getter(g, swift_inner_type.clone(), inner)),
                setter
                    .as_ref()
                    .map(|s| map_vec_setter(s, swift_inner_type.clone(), inner)),
                format!("[{}]", swift_inner_type),
            )
        }

        FieldWrapper {
            wrapper_type: WrapperType::Option(inner),
            setter,
            getter,
            ..
        } => {
            let swift_inner_type = match inner.deref() {
                WrapperType::IntegerNumber(name)
                | WrapperType::FloatingPointNumber(name)
                | WrapperType::Struct(name)
                | WrapperType::Enum(name) => name.clone(),
                WrapperType::Bool => "Bool".to_string(),
                WrapperType::String => "String".to_string(),
                WrapperType::Vec(_) => panic!("Vec in option not supported as property"),
                WrapperType::Result(_) => panic!("Result in option not supported as property"),
                WrapperType::Option(_) => panic!("Nested options not supported as property"),
                WrapperType::UnitExpr => panic!("Empty expression cannot be a swift property"),
                WrapperType::Trait(_) => panic!("Trait not supported as property"),
            };
            (
                getter
                    .as_ref()
                    .map(|g| map_option_getter(g, swift_inner_type.clone(), inner)),
                setter
                    .as_ref()
                    .map(|s| map_option_setter(s, swift_inner_type.clone(), inner)),
                format!("{}?", swift_inner_type),
            )
        }

        FieldWrapper {
            wrapper_type: WrapperType::Result(_) | WrapperType::Trait(_) | WrapperType::UnitExpr,
            ..
        } => panic!("Unsupported wrapper type for struct field"),
    };

    let field_name = &field.field_name;
    match (getter, setter) {
        (Some(getter), Some(setter)) => {
            format!(
                r#"
    public var {field_name}: {swift_field_type} {{
{getter}
{setter}
    }}
"#,
            )
        }
        (Some(getter), None) => {
            format!(
                r#"
    public var {field_name}: {swift_field_type} {{
{getter}
    }}
"#,
            )
        }
        (None, Some(setter)) => {
            format!(
                r#"
    public var {field_name}: {swift_field_type} {{
{setter}
    }}
"#,
            )
        }
        (None, None) => String::new(),
    }
}

fn map_custom_getter(Getter { extern_fn_name, .. }: &Getter, field_type: impl Display) -> String {
    format!(
        r#"
        get {{
            let ptr = {extern_fn_name}(self.rawPtr())
            return {field_type}(ptr!)
        }}"#,
    )
}

fn map_custom_setter(Setter { extern_fn_name, .. }: &Setter) -> String {
    format!(
        r#"
        set {{
            {extern_fn_name}(self.rawPtr(), newValue.rawPtr())
        }}"#,
    )
}

fn map_primitive_getter(Getter { extern_fn_name, .. }: &Getter) -> String {
    format!(
        r#"
        get {{
            return {extern_fn_name}(self.rawPtr())
        }}"#,
    )
}

fn map_primitive_setter(Setter { extern_fn_name, .. }: &Setter) -> String {
    format!(
        r#"
        set {{
            {extern_fn_name}(self.rawPtr(), newValue)
        }}"#,
    )
}

fn map_enum_getter(Getter { extern_fn_name, .. }: &Getter, enum_name: impl Display) -> String {
    format!(
        r#"
        get {{
            return {enum_name}(rawValue: Int32({extern_fn_name}(self.rawPtr()).rawValue))!
        }}"#,
    )
}

fn map_enum_setter(Setter { extern_fn_name, .. }: &Setter, enum_name: impl Display) -> String {
    format!(
        r#"
        set {{
            {extern_fn_name}(self.rawPtr(), CFfiModule.{enum_name}(rawValue: UInt32(newValue.rawValue)))
        }}"#,
    )
}

fn map_string_getter(Getter { extern_fn_name, .. }: &Getter) -> String {
    format!(
        r#"
        get {{
            let slice_ptr = {extern_fn_name}(self.rawPtr())
            let c_str_ptr = {SLICE_GET_PTR_FN_NAME}(slice_ptr)
            let c_str_len = {SLICE_GET_LEN_FN_NAME}(slice_ptr)
            let typed_pointer = c_str_ptr!.assumingMemoryBound(to: UInt8.self)
            let bytes: UnsafeBufferPointer<UInt8> = UnsafeBufferPointer(start: typed_pointer, count: Int(c_str_len))
            let result = String(bytes: bytes, encoding: .utf8)!
            {SLICE_DROP_FN_NAME}(slice_ptr)
            return result
        }}"#,
    )
}

fn map_string_setter(Setter { extern_fn_name, .. }: &Setter) -> String {
    format!(
        r#"
        set {{
            let str_ptr = newValue.utf8CString.withUnsafeBufferPointer({{ ptr in return UnsafeMutableRawPointer(mutating: ptr.baseAddress!) }})
            {extern_fn_name}(self.rawPtr(), str_ptr, UInt32(newValue.utf8CString.count))
        }}"#,
    )
}

fn map_vec_getter(
    Getter { extern_fn_name, .. }: &Getter,
    _field_type: impl Display,
    inner: &WrapperType,
) -> String {
    let inner_name = match inner {
        WrapperType::String => "String".to_string(),
        _ => inner.name(),
    };
    let vec_class_name = format!("Rust{inner_name}Vec");

    format!(
        r#"
        get {{
            let ptr = {extern_fn_name}(self.rawPtr())
            let rust_vec = {vec_class_name}(ptr!)
            let swift_array = rust_vec.toSwift()
            let _ = rust_vec.leak()
            return swift_array
        }}"#,
    )
}

fn map_vec_setter(
    Setter { extern_fn_name, .. }: &Setter,
    _field_type: impl Display,
    inner: &WrapperType,
) -> String {
    let inner_name = match inner {
        WrapperType::String => "String".to_string(),
        _ => inner.name(),
    };
    let vec_class_name = format!("Rust{inner_name}Vec");

    format!(
        r#"
        set {{
            let rust_vec = {vec_class_name}.fromSwift(newValue)
            {extern_fn_name}(self.rawPtr(), rust_vec.rawPtr())
        }}"#,
    )
}
fn map_option_getter(
    Getter { extern_fn_name, .. }: &Getter,
    _field_type: impl Display,
    inner: &WrapperType,
) -> String {
    let inner_name = inner.name();
    let option_class_name = format!("Rust{inner_name}Option");

    format!(
        r#"
        get {{
            let ptr = {extern_fn_name}(self.rawPtr())
            let rust_option = {option_class_name}(ptr!)
            defer {{ let _  = rust_option.leak() }}
            return rust_option.toSwift()
        }}"#,
    )
}

fn map_option_setter(
    Setter { extern_fn_name, .. }: &Setter,
    _field_type: impl Display,
    inner: &WrapperType,
) -> String {
    let inner_name = inner.name();
    let option_class_name = format!("Rust{inner_name}Option");

    format!(
        r#"
        set {{
            let rust_option = {option_class_name}.fromSwift(newValue)
            {extern_fn_name}(self.rawPtr(), rust_option.rawPtr())
        }}"#,
    )
}
