use core::panic;
use std::collections::HashSet;
use std::fmt::Display;

use quote::{format_ident, quote};
use syn::ItemStruct;

use crate::EXPORTED_SYMBOLS_PREFIX;
use crate::translator::{NoWrapperErr, path_to_wrapper_type};
use crate::wrapper::*;

pub fn translate_struct(item_struct: ItemStruct) -> Result<Wrapper, NoWrapperErr> {
    let class_name = &item_struct.ident;

    register_struct_type(class_name.to_string());

    Ok(Wrapper {
        original_definition: quote! {#item_struct},
        parsed: ParsedWrapper::Struct(StructWrapper {
            name: class_name.clone(),
            fields: fields_wrappers(&item_struct)?,
            default_constructor: default_constructor(&item_struct),
            partial_eq: partial_eq_impl(&item_struct),
            drop_ext_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}__{class_name}__drop"),
            clone_ext_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}__{class_name}__clone"),
            original_item_struct: item_struct,
        }),
        reusable_wrappers: HashSet::new(),
    })
}

fn fields_wrappers(item_struct: &ItemStruct) -> Result<Vec<FieldWrapper>, NoWrapperErr> {
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
                let wrapper_type = path_to_wrapper_type(&path.path)?;
                Ok(FieldWrapper {
                    field_name,
                    field_type: field.ty.clone(),
                    wrapper_type,
                    setter,
                    getter,
                })
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
            extern_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}__{class_name}__get_{field_name}",),
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
            extern_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}__{class_name}__set_{field_name}",),
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

pub fn partial_eq_impl(item_struct: &ItemStruct) -> Option<PartialEqImpl> {
    let mut partial_eq_impl = None;

    let class_name = &item_struct.ident;

    for attr in &item_struct.attrs {
        if attr.path().is_ident("derive") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("PartialEq") {
                    partial_eq_impl = Some(class_name_to_partial_eq_impl(class_name));
                }
                Ok(())
            });
        }
    }

    partial_eq_impl
}

pub(crate) fn class_name_to_partial_eq_impl(class_name: impl Display) -> PartialEqImpl {
    PartialEqImpl {
        extern_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}__{class_name}__partial_eq"),
        eq_fn_name: format_ident!("{class_name}__partial_eq"),
    }
}

pub fn default_constructor(item_struct: &ItemStruct) -> Option<DefaultConstructor> {
    let mut default_constructor = None;

    let class_name = &item_struct.ident;

    for attr in &item_struct.attrs {
        if attr.path().is_ident("derive") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("Default") {
                    default_constructor = Some(class_name_to_default_constructor(class_name));
                }
                Ok(())
            });
        }
    }

    default_constructor
}

pub(crate) fn class_name_to_default_constructor(class_name: impl Display) -> DefaultConstructor {
    DefaultConstructor {
        extern_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}__{class_name}__default"),
        constructor_name: format_ident!("{class_name}__default"),
    }
}
