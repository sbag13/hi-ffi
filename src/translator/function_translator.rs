use quote::{ToTokens, quote};
use std::collections::HashSet;
use std::ops::Deref;
use syn::{FnArg, GenericArgument, ItemFn, PathSegment, Type};

use crate::EXPORTED_SYMBOLS_PREFIX;
use crate::translator::{NoWrapperErr, map_wrapper_to_reusable};
use crate::wrapper::*;

impl FunctionWrapper {
    pub(crate) fn reusable_wrappers(&self) -> HashSet<ReusableWrapper> {
        let mut reusable_wrappers = self
            .args
            .iter()
            .flat_map(|arg_wrapper| map_wrapper_to_reusable(&arg_wrapper.wrapper_type))
            .collect::<std::collections::HashSet<_>>();

        // Ensure vec reusable wrapper is generated for return vecs too
        if let Some(return_wrapper) = &self.return_wrapper {
            reusable_wrappers.extend(map_wrapper_to_reusable(&return_wrapper.wrapper_type));
        }
        reusable_wrappers
    }
}

pub fn translate_function(item_fn: ItemFn) -> Result<Wrapper, NoWrapperErr> {
    let fn_wrapper = fn_wrapper_from_sig(&item_fn.sig)?;

    Ok(Wrapper {
        original_definition: quote! {#item_fn},
        reusable_wrappers: fn_wrapper.reusable_wrappers(),
        parsed: ParsedWrapper::Function(fn_wrapper),
    })
}

// It ignores the receiver argument
pub(crate) fn fn_wrapper_from_sig(sig: &syn::Signature) -> Result<FunctionWrapper, NoWrapperErr> {
    let fn_name = &sig.ident;
    let args_wrappers = sig
        .inputs
        .iter()
        .filter_map(|arg| map_arg(arg).transpose())
        .collect::<Result<Vec<_>, _>>()?;

    let return_wrapper = return_wrapper(&sig.output)?;

    Ok(FunctionWrapper {
        name: fn_name.clone(),
        extern_function_name: format!("{EXPORTED_SYMBOLS_PREFIX}_{fn_name}"),
        args: args_wrappers,
        return_wrapper,
    })
}

pub fn return_wrapper(
    output: &syn::ReturnType,
) -> Result<Option<FunctionReturnWrapper>, NoWrapperErr> {
    match output {
        syn::ReturnType::Default => Ok(None),
        syn::ReturnType::Type(_, ty) => {
            // Support simple idents and generic types like Vec<T>
            if let syn::Type::Path(path) = ty.deref() {
                if let Some(ident) = path.path.get_ident() {
                    let wrapper_type = ident.to_string().as_str().parse()?;
                    Ok(Some(FunctionReturnWrapper {
                        wrapper_type,
                        return_type: ty.deref().clone(),
                    }))
                } else {
                    // Handle non-trivial paths, e.g., Vec<T>
                    match path.path.segments.first() {
                        Some(segment) => {
                            if let Some(vec_wrapper_type) = segment_as_vec(segment)? {
                                Ok(Some(FunctionReturnWrapper {
                                    wrapper_type: vec_wrapper_type,
                                    return_type: ty.deref().clone(),
                                }))
                            } else if let Some(result_wrapper_type) = segment_as_result(segment)? {
                                Ok(Some(FunctionReturnWrapper {
                                    wrapper_type: result_wrapper_type,
                                    return_type: ty.deref().clone(),
                                }))
                            } else if let Some(option_wrapper_type) = segment_as_opt(segment)? {
                                Ok(Some(FunctionReturnWrapper {
                                    wrapper_type: option_wrapper_type,
                                    return_type: ty.deref().clone(),
                                }))
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

fn segment_as_vec(vec_segment: &PathSegment) -> Result<Option<WrapperType>, NoWrapperErr> {
    match vec_segment.ident.to_string().as_str() {
        "Vec" => {
            let inner_wrapper_type: WrapperType =
                parse_generic_single_inner_type_from_segment(vec_segment)?;

            match inner_wrapper_type {
                WrapperType::Vec(_) => {
                    panic!("Nested vectors are not supported as return types")
                }
                inner => Ok(Some(WrapperType::Vec(Box::new(inner)))),
            }
        }
        _ => Ok(None),
    }
}

fn segment_as_opt(opt_segment: &PathSegment) -> Result<Option<WrapperType>, NoWrapperErr> {
    match opt_segment.ident.to_string().as_str() {
        "Option" => {
            let inner_wrapper_type: WrapperType =
                parse_generic_single_inner_type_from_segment(opt_segment)?;
            match inner_wrapper_type {
                WrapperType::Option(_) => {
                    panic!("Nested options are not supported as return types")
                }
                inner => Ok(Some(WrapperType::Option(Box::new(inner)))),
            }
        }
        _ => Ok(None),
    }
}

fn segment_as_result(result_segment: &PathSegment) -> Result<Option<WrapperType>, NoWrapperErr> {
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
                                if let Some(vec_wrapper_type) = segment_as_vec(segment)? {
                                    vec_wrapper_type
                                } else {
                                    ok_path.to_token_stream().to_string().as_str().parse()?
                                }
                            } else {
                                panic!("No segment found in Result Ok return type")
                            }
                        }
                        GenericArgument::Type(Type::Tuple(t)) if t.elems.empty_or_trailing() => {
                            WrapperType::UnitExpr
                        }
                        _ => {
                            panic!("Result Ok inner return type must be a path")
                        }
                    }
                }
                _ => panic!("Result return type arguments not supported"),
            };
            Ok(Some(WrapperType::Result(Box::new(ok_wrapper_type))))
        }
        _ => Ok(None),
    }
}

pub fn map_arg(arg: &FnArg) -> Result<Option<FunctionArgWrapper>, NoWrapperErr> {
    match arg {
        syn::FnArg::Receiver(_) => Ok(None), // ignore receiver
        syn::FnArg::Typed(pat_type) => {
            let pat = &pat_type.pat;
            let ty = &pat_type.ty;
            let arg_name = match pat.deref() {
                syn::Pat::Ident(ident) => ident.ident.clone(),
                _ => panic!("Only simple argument names are supported"),
            };
            let wrapper_type = if let syn::Type::Path(path) = ty.deref() {
                if let Some(ident) = path.path.get_ident() {
                    ident.to_string().as_str().parse()?
                } else {
                    match path.path.segments.first() {
                        Some(segment) => match segment.ident.to_string().as_str() {
                            "Vec" => {
                                let inner_wrapper_type: WrapperType =
                                    parse_generic_single_inner_type_from_segment(segment)?;
                                match inner_wrapper_type {
                                    WrapperType::Vec(_) => {
                                        panic!("Nested vectors are not supported")
                                    }
                                    WrapperType::Option(_) => {
                                        panic!("Vec of options not supported yet")
                                    }
                                    WrapperType::Result(_) => {
                                        panic!("Vec of results not supported yet")
                                    }
                                    inner => WrapperType::Vec(Box::new(inner)),
                                }
                            }
                            "Option" => {
                                let inner_wrapper_type: WrapperType =
                                    parse_generic_single_inner_type_from_segment(segment)?;

                                match inner_wrapper_type {
                                    WrapperType::Option(_) => {
                                        panic!("Nested options not supported yet")
                                    }
                                    WrapperType::Result(_) => {
                                        panic!("Optional Results not supported yet")
                                    }
                                    inner => WrapperType::Option(Box::new(inner)),
                                }
                            }
                            "Box" => parse_generic_single_inner_type_from_segment(segment)?,
                            _ => {
                                panic!("Unsupported function type: {:?}", segment.ident)
                            }
                        },
                        None => {
                            panic!("No segment found in function argument type")
                        }
                    }
                }
            } else {
                panic!("No path found in function argument type")
            };

            Ok(Some(FunctionArgWrapper {
                wrapper_type,
                arg_name,
                arg_type: ty.deref().clone(),
            }))
        }
    }
}

fn parse_generic_single_inner_type_from_segment(
    segment: &PathSegment,
) -> Result<WrapperType, NoWrapperErr> {
    match &segment.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            let Some(inner_arg) = args.args.first() else {
                panic!("No argument found in generic's arguments");
            };
            match inner_arg {
                GenericArgument::Type(Type::Path(inner_path)) => {
                    Ok(inner_path.to_token_stream().to_string().as_str().parse()?)
                }
                GenericArgument::Type(Type::TraitObject(type_trait_obj)) => {
                    if type_trait_obj.bounds.len() != 1 {
                        panic!(
                            "Only one trait bound is supported in trait objects for function arguments"
                        );
                    }
                    let first_bound = type_trait_obj.bounds.first().unwrap();

                    match first_bound {
                        syn::TypeParamBound::Trait(trait_bound) => {
                            let trait_ident = &trait_bound.path.segments.last().unwrap().ident;
                            // TODO try to find it in the registered traits
                            Ok(WrapperType::Trait(trait_ident.to_string()))
                        }
                        _ => panic!(
                            "Only trait bounds are supported in trait objects for function arguments"
                        ),
                    }
                }
                _ => panic!("Generic's inner arg type must be a path"),
            }
        }
        _ => panic!("Generic arguments not supported"),
    }
}
