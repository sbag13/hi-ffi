use std::fmt::Display;

use crate::wrapper::{
    base::{SLICE_DROP_FN_NAME, SLICE_GET_LEN_FN_NAME, SLICE_GET_PTR_FN_NAME},
    swift::function_definition::{
        compose_function_definition, map_args, map_header_declaration_args, map_return_type,
        ReturnTypes,
    },
    FieldWrapper, FieldWrapperType, Getter, ImplBlockWrapper, Setter, StructWrapper,
};
use quote::ToTokens;

use super::impl_block_wrapper::MethodWrapper;

pub const METHOD_DEFINITIONS_MARKER: &str = "// class method definitions";

pub fn gen_method_declarations_from_struct(struct_wrapper: &StructWrapper) -> String {
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

pub fn gen_method_declarations_from_impl_block(impl_block_wrapper: &ImplBlockWrapper) -> String {
    impl_block_wrapper
        .methods
        .iter()
        .filter(|&method| method.public)
        .map(gen_method_header)
        .collect::<Vec<_>>()
        .join("\n")
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
    impl_block_wrapper
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
        .collect::<Vec<_>>()
        .join("\n")
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
            wrapper_type: FieldWrapperType::String,
            setter,
            getter,
            ..
        } => (
            getter.as_ref().map(map_string_getter_as_extern_fn),
            setter.as_ref().map(map_string_setter_as_extern_fn),
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
    let default_constructor = gen_default_constructor(struct_wrapper);

    format!(
        r#"
    deinit {{
        {destructor_extern_fn}(self.rawPtr());
    }}
{default_constructor}
{props}
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
            setter,
            getter,
            field_type,
            ..
        } => (
            getter
                .as_ref()
                .map(|g| map_custom_getter(g, field_type.to_token_stream())),
            setter.as_ref().map(map_custom_setter),
        ),

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
