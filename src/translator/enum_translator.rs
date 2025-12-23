use core::panic;
use std::collections::HashSet;

use quote::{ToTokens, quote};
use syn::{ItemEnum, Variant};

use crate::wrapper::enum_wrapper::{EnumVariant, EnumWrapper};
use crate::wrapper::*;

pub fn translate_enum(item_enum: ItemEnum) -> Wrapper {
    let enum_name = &item_enum.ident;

    // Validate that enum has repr(C) attribute and only unit variants
    validate_c_like_enum(&item_enum);

    // Register the enum type for later detection in function return types
    crate::wrapper::function_wrapper::register_enum_type(enum_name.to_string());

    Wrapper {
        original_definition: quote! {#item_enum},
        parsed: ParsedWrapper::Enum(EnumWrapper {
            name: enum_name.clone(),
            variants: extract_variants(&item_enum),
        }),
        reusable_wrappers: HashSet::new(),
    }
}

fn validate_c_like_enum(item_enum: &ItemEnum) {
    // Check for repr(C) attribute
    let has_repr_c = item_enum.attrs.iter().any(|attr| {
        if attr.path().is_ident("repr") {
            let mut found_c = false;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("C") {
                    found_c = true;
                    return Ok(());
                }
                Ok(())
            });
            found_c
        } else {
            false
        }
    });

    if !has_repr_c {
        panic!(
            "Enum {} must have #[repr(C)] attribute to be compatible with C FFI",
            item_enum.ident
        );
    }

    // Check that all variants are unit variants (no fields)
    for variant in &item_enum.variants {
        match variant.fields {
            syn::Fields::Named(_) => {
                panic!(
                    "Enum {} variant {} has named fields. Only unit variants are supported for C-like enums",
                    item_enum.ident, variant.ident
                );
            }
            syn::Fields::Unnamed(_) => {
                panic!(
                    "Enum {} variant {} has unnamed fields. Only unit variants are supported for C-like enums",
                    item_enum.ident, variant.ident
                );
            }
            syn::Fields::Unit => {
                // This is what we want - unit variant
            }
        }
    }
}

fn extract_variants(item_enum: &ItemEnum) -> Vec<EnumVariant> {
    item_enum
        .variants
        .iter()
        .map(|variant| EnumVariant {
            name: variant.ident.clone(),
            discriminant: get_discriminant_as_string(variant),
        })
        .collect()
}

fn get_discriminant_as_string(variant: &Variant) -> Option<String> {
    variant
        .discriminant
        .as_ref()
        .map(|(_, expr)| expr.to_token_stream().to_string())
}
