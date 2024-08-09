use core::panic;
use std::fmt::Display;

use quote::{format_ident, quote};
use syn::ItemStruct;

use crate::{wrapper::*, EXPORTED_SYMBOLS_PREFIX};

pub fn translate_struct(item_struct: ItemStruct) -> Wrapper {
    let class_name = &item_struct.ident;

    Wrapper {
        original_definition: quote! {#item_struct},
        parsed: ParsedWrapper::Struct(StructWrapper {
            name: class_name.clone(),
            fields: fields_wrappers(&item_struct),
            default_constructor: default_constructor(&item_struct),
            drop_ext_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}${class_name}__drop"),
            clone_ext_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}${class_name}__clone"),
            original_item_struct: item_struct,
        }),
    }
}

fn fields_wrappers(item_struct: &ItemStruct) -> Vec<FieldWrapper> {
    let class_name = &item_struct.ident;
    item_struct
        .fields
        .iter()
        .map(|field| {
            let is_public = matches!(field.vis, syn::Visibility::Public(_));

            let field_name = field.ident.clone().expect("No ident found for a field");
            let field_attributes = extract_field_attributes(field);

            let getter = generate_getter(&field_attributes, class_name, &field_name, is_public);
            let setter = generate_setter(&field_attributes, class_name, &field_name, is_public);

            if let syn::Type::Path(path) = &field.ty {
                if let Some(ident) = path.path.get_ident() {
                    let wrapper_type = match ident.to_string().as_str() {
                        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64"
                        | "u128" | "f32" | "f64" | "bool" => FieldWrapperType::Primitive,
                        "String" => FieldWrapperType::String,
                        _custom_type => FieldWrapperType::Custom,
                    };

                    FieldWrapper {
                        field_name,
                        field_type: field.ty.clone(),
                        getter,
                        wrapper_type,
                        setter,
                    }
                } else {
                    panic!("No ident found")
                }
            } else {
                panic!("No path found")
            }
        })
        .collect()
}

fn generate_getter(
    attrs: &FieldAttributes,
    class_name: impl Display,
    field_name: impl Display,
    is_public: bool,
) -> Option<Getter> {
    if !attrs.skip_attr && (is_public || attrs.getter_attr) {
        Some(Getter {
            extern_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}${class_name}__get_{field_name}",),
            name: format_ident!("get_{field_name}"),
        })
    } else {
        None
    }
}

fn generate_setter(
    attrs: &FieldAttributes,
    class_name: impl Display,
    field_name: impl Display,
    is_public: bool,
) -> Option<Setter> {
    if !attrs.skip_attr && (is_public || attrs.setter_attr) {
        Some(Setter {
            extern_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}${class_name}__set_{field_name}",),
            name: format_ident!("set_{field_name}"),
        })
    } else {
        None
    }
}

struct FieldAttributes {
    pub getter_attr: bool,
    pub setter_attr: bool,
    pub skip_attr: bool,
}

fn extract_field_attributes(field: &syn::Field) -> FieldAttributes {
    let mut getter = false;
    let mut setter = false;
    let mut skip = false;

    field.attrs.iter().for_each(|attr| {
        if attr.path().is_ident("ffi") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("setter") {
                    setter = true;
                    return Ok(());
                }
                if meta.path.is_ident("getter") {
                    getter = true;
                    return Ok(());
                }
                if meta.path.is_ident("skip") {
                    skip = true;
                    return Ok(());
                }
                Err(meta.error("Unsupported attribute"))
            });
        }
    });

    FieldAttributes {
        getter_attr: getter,
        setter_attr: setter,
        skip_attr: skip,
    }
}

fn default_constructor(item_struct: &ItemStruct) -> Option<DefaultConstructor> {
    let mut default_constructor = None;

    let class_name = &item_struct.ident;

    for attr in &item_struct.attrs {
        if attr.path().is_ident("derive") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("Default") {
                    default_constructor = Some(DefaultConstructor {
                        extern_fn_name: format!(
                            "{EXPORTED_SYMBOLS_PREFIX}${class_name}__default",
                            class_name = class_name
                        ),
                        constructor_name: format_ident!("{class_name}__default"),
                    });
                    return Ok(());
                }
                Err(meta.error("unsupported attribute"))
            });
        }
    }

    default_constructor
}
