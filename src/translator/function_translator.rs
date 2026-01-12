use quote::{ToTokens, quote};
use std::ops::Deref;
use syn::{FnArg, GenericArgument, ItemFn, Type};

use crate::EXPORTED_SYMBOLS_PREFIX;
use crate::wrapper::*;

pub fn translate_function(item_struct: ItemFn) -> Wrapper {
    let fn_name = &item_struct.sig.ident;
    let args_wrappers = item_struct
        .sig
        .inputs
        .iter()
        .map(map_arg)
        .collect::<Vec<_>>();

    let mut reusable_wrappers = args_wrappers
        .iter()
        .filter_map(|arg_wrapper| match &arg_wrapper.wrapper_type {
            WrapperType::Vec(inner_wrapper_type) => {
                Some(ReusableWrapper::Vec(*inner_wrapper_type.clone()))
            }
            WrapperType::Result(inner_wrapper_type) => {
                Some(ReusableWrapper::Result(*inner_wrapper_type.clone()))
            }
            _ => None,
        })
        .collect::<std::collections::HashSet<_>>();

    let return_wrapper = return_wrapper(&item_struct.sig.output);

    // Ensure vec reusable wrapper is generated for return vecs too
    if let Some(return_wrapper) = &return_wrapper {
        match return_wrapper {
            FunctionReturnWrapper {
                wrapper_type: WrapperType::Vec(inner),
                ..
            } => {
                reusable_wrappers.insert(ReusableWrapper::Vec(*inner.clone()));
            }
            FunctionReturnWrapper {
                wrapper_type: WrapperType::Result(inner),
                ..
            } => {
                reusable_wrappers.insert(ReusableWrapper::Result(*inner.clone()));
            }
            _ => (),
        }
    }

    Wrapper {
        original_definition: quote! {#item_struct},
        parsed: ParsedWrapper::Function(FunctionWrapper {
            name: fn_name.clone(),
            extern_function_name: format!("{EXPORTED_SYMBOLS_PREFIX}_{fn_name}"),
            args_wrappers,
            return_wrapper,
        }),
        reusable_wrappers,
    }
}

pub fn return_wrapper(output: &syn::ReturnType) -> Option<FunctionReturnWrapper> {
    match output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => {
            // Support simple idents and generic types like Vec<T>
            if let syn::Type::Path(path) = ty.deref() {
                if let Some(ident) = path.path.get_ident() {
                    let wrapper_type = ident.to_string().as_str().parse().unwrap();
                    Some(FunctionReturnWrapper {
                        wrapper_type,
                        return_type: ty.deref().clone(),
                    })
                } else {
                    // Handle non-trivial paths, e.g., Vec<T>
                    match path.path.segments.first() {
                        Some(segment) => {
                            if let Some(vec_wrapper_type) = segment_as_vec(segment) {
                                Some(FunctionReturnWrapper {
                                    wrapper_type: vec_wrapper_type,
                                    return_type: ty.deref().clone(),
                                })
                            } else if let Some(result_wrapper_type) = segment_as_result(segment) {
                                Some(FunctionReturnWrapper {
                                    wrapper_type: result_wrapper_type,
                                    return_type: ty.deref().clone(),
                                })
                            } else {
                                panic!("Unsupported return type: {:?}", segment.ident);
                            }
                        }
                        None => panic!("No segment found in return type"),
                    }
                }
            } else {
                panic!("No path found in return type")
            }
        }
    }
}

fn segment_as_vec(vec_segment: &syn::PathSegment) -> Option<WrapperType> {
    match vec_segment.ident.to_string().as_str() {
        "Vec" => {
            let inner_wrapper_type: WrapperType = match &vec_segment.arguments {
                syn::PathArguments::AngleBracketed(args) => {
                    let Some(inner_arg) = args.args.first() else {
                        panic!("No argument found in Vec arguments");
                    };
                    match inner_arg {
                        GenericArgument::Type(Type::Path(inner_path)) => inner_path
                            .to_token_stream()
                            .to_string()
                            .as_str()
                            .parse()
                            .unwrap(),
                        _ => panic!("Vector inner arg type must be a path"),
                    }
                }
                _ => panic!("Vec arguments not supported"),
            };

            match inner_wrapper_type {
                WrapperType::Vec(_) => {
                    panic!("Nested vectors are not supported")
                }
                inner => Some(WrapperType::Vec(Box::new(inner))),
            }
        }
        _ => None,
    }
}

fn segment_as_result(result_segment: &syn::PathSegment) -> Option<WrapperType> {
    match result_segment.ident.to_string().as_str() {
        "Result" => {
            let ok_wrapper_type: WrapperType = match &result_segment.arguments {
                syn::PathArguments::AngleBracketed(args) => {
                    let Some(ok_arg) = args.args.first() else {
                        panic!("No argument found in Result return type");
                    };
                    match ok_arg {
                        GenericArgument::Type(Type::Path(ok_path)) => {
                            if let Some(segment) = ok_path.path.segments.first() {
                                if let Some(vec_wrapper_type) = segment_as_vec(segment) {
                                    vec_wrapper_type
                                } else {
                                    ok_path
                                        .to_token_stream()
                                        .to_string()
                                        .as_str()
                                        .parse()
                                        .unwrap()
                                }
                            } else {
                                panic!("No segment found in Result Ok return type")
                            }
                        }
                        _ => {
                            panic!("Result Ok inner return type must be a path")
                        }
                    }
                }
                _ => panic!("Result return type arguments not supported"),
            };
            Some(WrapperType::Result(Box::new(ok_wrapper_type)))
        }
        _ => None,
    }
}

pub fn map_arg(arg: &FnArg) -> FunctionArgWrapper {
    match arg {
        syn::FnArg::Receiver(_) => panic!("Receiver argument is not supported"),
        syn::FnArg::Typed(pat_type) => {
            let pat = &pat_type.pat;
            let ty = &pat_type.ty;
            let arg_name = match pat.deref() {
                syn::Pat::Ident(ident) => ident.ident.clone(),
                _ => panic!("Only simple argument names are supported"),
            };
            let wrapper_type = if let syn::Type::Path(path) = ty.deref() {
                if let Some(ident) = path.path.get_ident() {
                    ident.to_string().as_str().parse().unwrap()
                } else {
                    match path.path.segments.first() {
                        Some(segment) => match segment.ident.to_string().as_str() {
                            "Vec" => {
                                let inner_wrapper_type: WrapperType = match &segment.arguments {
                                    syn::PathArguments::AngleBracketed(args) => {
                                        let Some(inner_arg) = args.args.first() else {
                                            panic!("No argument found in Vec arguments");
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
                                            _ => panic!("Vector inner arg type must be a path"),
                                        }
                                    }
                                    _ => panic!("Vec arguments not supported"),
                                };

                                match inner_wrapper_type {
                                    WrapperType::Vec(_) => {
                                        panic!("Nested vectors are not supported")
                                    }
                                    inner => WrapperType::Vec(Box::new(inner)),
                                }
                            }
                            _ => {
                                panic!("Unsupported type: {:?}", segment.ident)
                            }
                        },
                        None => {
                            panic!("No segment found")
                        }
                    }
                }
            } else {
                panic!("No path found")
            };

            FunctionArgWrapper {
                wrapper_type,
                arg_name,
                arg_type: ty.deref().clone(),
            }
        }
    }
}
