use std::ops::Deref;

use quote::quote;
use syn::{FnArg, ItemFn};

use crate::{wrapper::*, EXPORTED_SYMBOLS_PREFIX};

pub fn translate_function(item_struct: ItemFn) -> Wrapper {
    let fn_name = &item_struct.sig.ident;
    let args_wrappers = item_struct
        .sig
        .inputs
        .iter()
        .map(map_arg)
        .collect::<Vec<_>>();

    let return_wrapper = return_wrapper(&item_struct.sig.output);

    Wrapper {
        original_definition: quote! {#item_struct},
        parsed: ParsedWrapper::Function(FunctionWrapper {
            name: fn_name.clone(),
            extern_function_name: format!("{EXPORTED_SYMBOLS_PREFIX}${fn_name}"),
            args_wrappers,
            return_wrapper,
        }),
    }
}

fn return_wrapper(output: &syn::ReturnType) -> Option<FunctionReturnWrapper> {
    match output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => {
            if let syn::Type::Path(path) = ty.deref() {
                if let Some(ident) = path.path.get_ident() {
                    let wrapper_type = match ident.to_string().as_str() {
                        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64"
                        | "u128" | "f32" | "f64" | "bool" => FunctionReturnWrapperType::Primitive,
                        "String" => FunctionReturnWrapperType::String,
                        _ => panic!("Unsupported type: {}", ident),
                    };

                    Some(FunctionReturnWrapper {
                        wrapper_type,
                        return_type: ty.deref().clone(),
                    })
                } else {
                    panic!("No ident found in return type")
                }
            } else {
                panic!("No path found in return type")
            }
        }
    }
}

fn map_arg(arg: &FnArg) -> FunctionArgWrapper {
    match arg {
        syn::FnArg::Receiver(_) => panic!("Receiver argument is not supported"),
        syn::FnArg::Typed(pat_type) => {
            let pat = &pat_type.pat;
            let ty = &pat_type.ty;
            let arg_name = match pat.deref() {
                syn::Pat::Ident(ident) => ident.ident.clone(),
                _ => panic!("Only simple argument names are supported"),
            };
            if let syn::Type::Path(path) = ty.deref() {
                if let Some(ident) = path.path.get_ident() {
                    let wrapper_type = match ident.to_string().as_str() {
                        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64"
                        | "u128" | "f32" | "f64" | "bool" => FunctionArgWrapperType::Primitive,
                        "String" => FunctionArgWrapperType::String,
                        _ => panic!("Unsupported type: {}", ident),
                    };

                    FunctionArgWrapper {
                        wrapper_type,
                        arg_name,
                        arg_type: ty.deref().clone(),
                    }
                } else {
                    panic!("No ident found")
                }
            } else {
                panic!("No path found")
            }
        }
    }
}
