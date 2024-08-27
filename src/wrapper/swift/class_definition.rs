use std::fmt::Display;

use super::*;
use crate::wrapper::base::{SLICE_DROP_FN_NAME, SLICE_GET_LEN_FN_NAME, SLICE_GET_PTR_FN_NAME};
use quote::ToTokens;

pub fn gen_class_header(struct_wrapper: &StructWrapper) -> String {
    let destructor_extern_fn = &struct_wrapper.drop_ext_fn_name;
    let getters_and_setters = gen_getters_and_setters_externs(struct_wrapper);
    let default_constructor = gen_default_constructor_ext(struct_wrapper);

    format!(
        r#"
void {destructor_extern_fn}(void*);
{getters_and_setters}
{default_constructor}
"#
    )
}

fn gen_default_constructor_ext(struct_wrapper: &StructWrapper) -> String {
    if let Some(default_constructor) = struct_wrapper.default_constructor.as_ref() {
        let default_constructor_ext_fn_name = &default_constructor.extern_fn_name;

        format!(
            r#"
void* {default_constructor_ext_fn_name}();
"#
        )
    } else {
        String::new()
    }
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
            wrapper_type: FieldWrapperType::Primitive,
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
            wrapper_type: FieldWrapperType::Custom,
            ..
        } => {
            (None, None) // TODO
        }
        FieldWrapper {
            wrapper_type: FieldWrapperType::String,
            setter,
            getter,
            ..
        } => (
            getter.as_ref().map(|g| map_string_getter_as_extern_fn(g)),
            setter.as_ref().map(|g| map_string_setter_as_extern_fn(g)),
        ),
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

fn map_primitive_setter_as_extern_fn(
    Setter { extern_fn_name, .. }: &Setter,
    field_type: impl Display,
) -> String {
    format!("void {extern_fn_name}(void*, {field_type});")
}

pub fn gen_class_definition(struct_wrapper: &StructWrapper) -> String {
    let class_name = &struct_wrapper.name;
    let destructor_extern_fn = &struct_wrapper.drop_ext_fn_name;
    let props = gen_props(struct_wrapper);
    let default_constructor = gen_default_constructor(struct_wrapper);

    format!(
        r#"
public class {class_name}: Opaque {{
    deinit {{
        {destructor_extern_fn}(self.rawPtr());
    }}
{default_constructor}
{props}
}}
"#
    )
}

fn gen_default_constructor(struct_wrapper: &StructWrapper) -> String {
    if let Some(default_constructor) = struct_wrapper.default_constructor.as_ref() {
        let default_constructor_ext_fn_name = &default_constructor.extern_fn_name;

        format!(
            r#"
    public convenience init() {{
        self.init({default_constructor_ext_fn_name}())
    }}
"#
        )
    } else {
        String::new()
    }
}

fn gen_props(struct_wrapper: &StructWrapper) -> String {
    struct_wrapper.fields.iter().map(gen_property).collect()
}

fn gen_property(field: &FieldWrapper) -> String {
    let (getter, setter) = match field {
        FieldWrapper {
            wrapper_type: FieldWrapperType::Primitive,
            setter,
            getter,
            ..
        } => (
            getter.as_ref().map(map_primitive_getter),
            setter.as_ref().map(map_primitive_setter),
        ),
        FieldWrapper {
            wrapper_type: FieldWrapperType::Custom,
            ..
        } => {
            (None, None) // TODO
        }
        FieldWrapper {
            wrapper_type: FieldWrapperType::String,
            setter,
            getter,
            ..
        } => (
            getter.as_ref().map(map_string_getter),
            setter.as_ref().map(map_string_setter),
        ),
    };

    let field_name = &field.field_name;
    let field_type = &field.field_type.to_token_stream().to_string();
    match (getter, setter) {
        (Some(getter), Some(setter)) => {
            format!(
                r#"
    public var {field_name}: {field_type} {{
{getter}
{setter}
    }}
"#,
            )
        }
        (Some(getter), None) => {
            format!(
                r#"
    public var {field_name}: {field_type} {{
{getter}
    }}
"#,
            )
        }
        (None, Some(setter)) => {
            format!(
                r#"
    public var {field_name}: {field_type} {{
{setter}
    }}
"#,
            )
        }
        (None, None) => String::new(),
    }
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
