use core::panic;
use std::collections::HashSet;
use std::fmt::Display;

use quote::{ToTokens, format_ident, quote};
use syn::{GenericArgument, ItemStruct, Type};

use crate::EXPORTED_SYMBOLS_PREFIX;
use crate::wrapper::*;

pub fn translate_struct(item_struct: ItemStruct) -> Wrapper {
    let class_name = &item_struct.ident;

    register_struct_type(class_name.to_string());

    Wrapper {
        original_definition: quote! {#item_struct},
        parsed: ParsedWrapper::Struct(StructWrapper {
            name: class_name.clone(),
            fields: fields_wrappers(&item_struct),
            default_constructor: default_constructor(&item_struct),
            drop_ext_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}__{class_name}__drop"),
            clone_ext_fn_name: format!("{EXPORTED_SYMBOLS_PREFIX}__{class_name}__clone"),
            original_item_struct: item_struct,
        }),
        reusable_wrappers: HashSet::new(),
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
                        _custom_type => FieldWrapperType::Custom(ident.to_string()),
                    };

                    FieldWrapper {
                        field_name,
                        field_type: field.ty.clone(),
                        getter,
                        wrapper_type,
                        setter,
                    }
                } else {
                    // Handle non-trivial paths, e.g., Vec<T>, Option<T>
                    match path.path.segments.first() {
                        Some(segment) => match segment.ident.to_string().as_str() {
                            "Vec" => {
                                let inner_wrapper_type: WrapperType = match &segment.arguments {
                                    syn::PathArguments::AngleBracketed(args) => {
                                        let Some(inner_arg) = args.args.first() else {
                                            panic!("No argument found in Vec return type");
                                        };
                                        match inner_arg {
                                            GenericArgument::Type(Type::Path(inner_path)) => {
                                                inner_path
                                                    .to_token_stream()
                                                    .to_string()
                                                    .as_str()
                                                    .parse()
                                                    .unwrap()
                                            }
                                            _ => panic!("Vector inner return type must be a path"),
                                        }
                                    }
                                    _ => panic!("Vec return type arguments not supported"),
                                };

                                FieldWrapper {
                                    field_name,
                                    field_type: field.ty.clone(),
                                    wrapper_type: FieldWrapperType::Vec(inner_wrapper_type),
                                    setter,
                                    getter,
                                }
                            }
                            "Option" => {
                                let inner_wrapper_type: WrapperType = match &segment.arguments {
                                    syn::PathArguments::AngleBracketed(args) => {
                                        let Some(inner_arg) = args.args.first() else {
                                            panic!("No argument found in Option field type");
                                        };
                                        match inner_arg {
                                            GenericArgument::Type(Type::Path(inner_path)) => {
                                                inner_path
                                                    .to_token_stream()
                                                    .to_string()
                                                    .as_str()
                                                    .parse()
                                                    .unwrap()
                                            }
                                            _ => panic!("Option inner field type must be a path"),
                                        }
                                    }
                                    _ => panic!("Option field type arguments not supported"),
                                };

                                FieldWrapper {
                                    field_name,
                                    field_type: field.ty.clone(),
                                    wrapper_type: FieldWrapperType::Option(Box::new(inner_wrapper_type)),
                                    setter,
                                    getter,
                                }
                            }
                            _ => panic!("Unsupported field type: {:?}", segment.ident),
                        },
                        None => {
                            panic!("No segment found in a field wrapper")
                        }
                    }
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

fn default_constructor(item_struct: &ItemStruct) -> Option<DefaultConstructor> {
    let mut default_constructor = None;

    let class_name = &item_struct.ident;

    for attr in &item_struct.attrs {
        if attr.path().is_ident("derive") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("Default") {
                    default_constructor = Some(DefaultConstructor {
                        extern_fn_name: format!(
                            "{EXPORTED_SYMBOLS_PREFIX}__{class_name}__default",
                            class_name = class_name
                        ),
                        constructor_name: format_ident!("{class_name}__default"),
                    });
                }
                Ok(())
            });
        }
    }

    default_constructor
}
